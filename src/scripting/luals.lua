local parser = require "parser"
local guide = parser.guide
local files = require "files"
local paths = require "workspace.require-path"

local function field(table_node, name)
    if not table_node or table_node.type ~= "table" then
        return nil
    end

    for _, entry in ipairs(table_node) do
        local key = entry.type == "tablefield" and entry.field[1]
            or entry.type == "tableindex" and entry.index.type == "string" and entry.index[1]

        if key == name then
            return entry.value
        end
    end
end

local function resolve(node, visited)
    visited = visited or {}

    if not node or visited[node] then
        return nil
    end

    visited[node] = true

    if node.type == "getlocal" then
        return resolve(node.node.value, visited)
    end

    if node.type == "select" and node.sindex == 1 then
        return resolve(node.vararg, visited)
    end

    if node.type == "getfield" or node.type == "getindex" then
        local name = node.field and node.field[1] or node.index and node.index.type == "string" and node.index[1]
        local table_node = resolve(node.node, visited)

        if table_node and table_node.type == "table" then
            return resolve(field(table_node, name), visited)
        end

        return node
    end

    if node.type == "call" and node.node.type == "getglobal" and node.node[1] == "require"
        and node.args and node.args[1] and node.args[1].type == "string" and node.args[1][1] ~= "kinematic" then
        local uri = paths.findUrisByRequireName(guide.getUri(node), node.args[1][1])[1]
        local text = uri and files.getOriginText(uri)

        if not text or visited[uri] then
            return nil
        end

        visited[uri] = true

        local state = parser.compile(text, "Lua", "Lua 5.4")

        if not state or not state.ast then
            return nil
        end

        state.uri = uri
        state.ast.uri = uri

        for _, action in ipairs(state.ast) do
            if action.type == "return" then
                return resolve(action[1], visited)
            end
        end

        return nil
    end

    return node
end

local function api_call(call, name)
    if not call or call.type ~= "call" then
        return false
    end

    local callee = call.node

    if callee.type == "getlocal" then
        callee = resolve(callee)
    end

    if not callee or callee.type ~= "getfield" or callee.field[1] ~= name then
        return false
    end

    local api = resolve(callee.node)

    if api and api.type == "getglobal" and api[1] == "k" then
        return true
    end

    return api and api.type == "call" and api.node.type == "getglobal" and api.node[1] == "require"
        and api.args and api.args[1] and api.args[1][1] == "kinematic"
end

local function value_type(node)
    node = resolve(node)

    if not node then
        return "any"
    end

    if node.type == "integer" or node.type == "number" then
        return "number"
    end

    if node.type == "boolean" then
        return "boolean"
    end

    if node.type == "string" then
        return "string"
    end

    for name, class in pairs { vec2 = "Vec2", vec3 = "Vec3", color = "Color", quat = "Quaternion", axis_angle = "Quaternion", quad = "Quad" } do
        if api_call(node, name) then
            return "Kinematic." .. class
        end
    end

    return "any"
end

local function class_name(call)
    local uri = guide.getUri(call)
    local hash = 0

    for index = 1, #uri do
        hash = (hash * 31 + uri:byte(index)) % 2147483647
    end

    local parent = call.parent

    if parent and parent.type == "select" then
        parent = parent.parent
    end

    local name = call.start

    if parent and parent.type == "local" then
        local ordinal = 0
        guide.eachSourceType(guide.getRoot(call), "local", function(source)
            if source[1] == parent[1] and source.start <= parent.start then
                ordinal = ordinal + 1
            end
        end)

        name = parent[1] .. "_" .. ordinal
    end

    return ("Kinematic.Custom_%d_%s"):format(hash, name)
end

local function member(name)
    local keyword = (" and break do else elseif end false for function goto if in local nil not or repeat return then true until while "):find(" " .. name .. " ", 1, true)

    if name:match("^[%a_][%w_]*$") and not keyword then
        return name, "." .. name
    end

    local indexed = ("[%q]"):format(name)

    return indexed, indexed
end

local function property_type(schema)
    local choices = resolve(field(schema, "choices"))

    if choices and choices.type == "table" then
        local values = {}

        for _, entry in ipairs(choices) do
            local value = resolve(entry.value)

            if value and value.type == "string" then
                values[#values + 1] = ("%q"):format(value[1])
            end
        end

        if #values > 0 then
            return table.concat(values, "|")
        end
    end

    return value_type(field(schema, "default"))
end

-- Describes component fields without executing project code.
function OnSetText(uri, text)
    if not uri:match("%.lua$") or uri:find("/kinematic/cache/luals/", 1, true)
        or not (text:find("object", 1, true) or text:find("component", 1, true)) then
        return nil
    end

    local state = parser.compile(text, "Lua", "Lua 5.4")

    if not state or not state.ast then
        return nil
    end

    state.uri = uri
    state.ast.uri = uri

    local diffs, classes = {}, {}

    local function source_text(node)
        local source_uri = guide.getUri(node)
        local source = source_uri == uri and text or files.getOriginText(source_uri)

        if not source then
            return nil
        end

        local parsed = source_uri == uri and state or parser.compile(source, "Lua", "Lua 5.4")

        if not parsed then
            return nil
        end

        local start = node.type == "function" and node.args and node.args.start or node.start
        local expression = source:sub(guide.positionToOffset(parsed, start) + 1, guide.positionToOffset(parsed, node.finish))

        if node.type == "function" and node.args and node.args[1] and node.args[1].type == "self" then
            expression = expression:gsub("^%(", #node.args > 1 and "(self, " or "(self", 1)
        end

        return node.type == "function" and "function" .. expression or expression
    end

    guide.eachSourceType(state.ast, "call", function(call)
        local component = api_call(call, "component")

        if not component and not api_call(call, "object_2d") and not api_call(call, "object_3d") then
            return
        end

        local options = call.args and resolve(call.args[2])

        if not options or options.type ~= "table" then
            return
        end

        local class = class_name(call)
        local parents = {}
        local definition, method_definitions, view_fields, option_fields = {}, {}, {}, {}
        local class_local = "_kinematic_" .. class:gsub("%W", "_")

        if component then
            parents[1] = "Kinematic.Component"

            for _, entry in ipairs(options) do
                local name = entry.type == "tablefield" and entry.field[1]
                    or entry.type == "tableindex" and entry.index.type == "string" and entry.index[1]
                local schema = resolve(entry.value)

                if name and field(schema, "default") then
                    local key, access = member(name)
                    local ty = property_type(schema)
                    local default = resolve(field(schema, "default"))
                    view_fields[#view_fields + 1] = ("---@field %s %s"):format(key, ty)
                    option_fields[#option_fields + 1] = ("---@field %s? %s"):format(key, ty)

                    if ty == "any" then
                        local source = default and source_text(default)

                        if source then
                            method_definitions[#method_definitions + 1] = ("%s%s = _kinematic_property(%s)"):format(class_local, access, source)
                        end
                    else
                        local property = ty:match("^Kinematic%.") and ty .. "Property" or "Kinematic.Property<" .. ty .. ">"
                        definition[#definition + 1] = ("---@field %s %s"):format(key, property)
                    end
                end
            end
        else
            local base = api_call(call, "object_3d") and "Kinematic.Script3D" or "Kinematic.Script2D"
            parents[1] = base
            definition[#definition + 1] = ("---@operator call(%sOptions): %s"):format(class, class)
            definition[#definition + 1] = ("---@field signal fun(self:%s, callback:fun(object:%s, frame:Kinematic.Frame)):Kinematic.Signal"):format(class, class)

            local methods = options

            if methods then
                for _, entry in ipairs(methods) do
                    local name = entry.type == "tablefield" and entry.field[1]
                        or entry.type == "tableindex" and entry.index.type == "string" and entry.index[1]
                    local method = resolve(entry.value)
                    local source = method and method.type == "function" and source_text(method)

                    if name and source and not name:match("^on_") then
                        local _, access = member(name)
                        local receiver = method.args and method.args[1]
                        local annotation = receiver and receiver[1] == "self"
                            and ("---@param %s %s\n"):format(receiver[1], class) or ""
                        method_definitions[#method_definitions + 1] = ("%s%s%s = %s"):format(annotation, class_local, access, source)
                    end
                end
            end

            local function annotate(method, name, start)
                local receiver = method.args and method.args[1]
                if not receiver then
                    return
                end
                local callback = name:match("^on_")
                local annotation = ("---@param %s %s\n"):format(receiver[1], class .. (callback and "View" or ""))
                if name == "on_draw" and method.args[2] then
                    annotation = annotation .. ("---@param %s %s\n"):format(method.args[2][1], api_call(call, "object_3d") and "Kinematic.Draw3D" or "Kinematic.Draw2D")
                elseif name == "on_update" and method.args[2] then
                    annotation = annotation .. ("---@param %s Kinematic.Frame\n"):format(method.args[2][1])
                end
                diffs[#diffs + 1] = {start = start, finish = start - 1, text = annotation}
            end

            for _, entry in ipairs(options) do
                local name = entry.type == "tablefield" and entry.field[1]
                local method = resolve(entry.value)
                if name and name:match("^on_") and method and method.type == "function" then
                    annotate(method, name, guide.positionToOffset(state, entry.start) + 1)
                end
            end

            for _, kind in ipairs {"setfield", "setmethod"} do
                guide.eachSourceType(state.ast, kind, function(assignment)
                    local owner = resolve(assignment.node)
                    local name = assignment.field and assignment.field[1] or assignment.method and assignment.method[1]
                    local method = assignment.value
                    if owner == call and name and method and method.type == "function" then
                        local start = guide.positionToOffset(state, assignment.start) + 1
                        -- Named function statements start before their assigned receiver.
                        local line = text:sub(1, start - 1):match("[^\n]*$") or ""
                        if line:match("^%s*function%s*$") then
                            start = start - #line
                        end
                        if kind == "setmethod" or name:match("^on_") then
                            annotate(method, name, start)
                        end
                        if not name:match("^on_") then
                            local _, access = member(name)
                            local source = source_text(method)
                            if source then
                                local annotation = kind == "setmethod" and ("---@param self %s\n"):format(class) or ""
                                method_definitions[#method_definitions + 1] = ("%s%s%s = %s"):format(annotation, class_local, access, source)
                            end
                        end
                    end
                end)
            end

            local components = resolve(field(options, "components"))

            local view_parents = {base .. "View"}
            local option_parents = {base .. "Options"}

            if components and components.type == "table" then
                for _, item in ipairs(components) do
                    local source = resolve(item.value)

                    if api_call(source, "component") then
                        parents[#parents + 1] = class_name(source)
                        view_parents[#view_parents + 1] = class_name(source) .. "View"
                        option_parents[#option_parents + 1] = class_name(source) .. "Options"
                    end
                end
            end
            classes[#classes + 1] = ("---@class %sView: %s\n\n---@class %sOptions: %s\n"):format(class, table.concat(view_parents, ","), class, table.concat(option_parents, ","))
        end

        if component then
            classes[#classes + 1] = ("---@class %sView\n%s\n\n---@class %sOptions\n%s\n"):format(class, table.concat(view_fields, "\n"), class, table.concat(option_fields, "\n"))
        end

        table.insert(definition, 1, ("\n---@class %s: %s"):format(class, table.concat(parents, ",")))
        definition[#definition + 1] = ("local %s = {}"):format(class_local)

        for _, method in ipairs(method_definitions) do
            definition[#definition + 1] = method
        end

        classes[#classes + 1] = table.concat(definition, "\n")

        local finish = guide.positionToOffset(state, call.finish) + 1
        local start = guide.positionToOffset(state, call.start) + 1
        diffs[#diffs + 1] = { start = start, finish = start - 1, text = "(" }
        diffs[#diffs + 1] = { start = finish, finish = finish - 1, text = (") --[[@as %s]]"):format(class) }
    end)

    if #classes > 0 then
        local start = #text + 1

        for _, action in ipairs(state.ast) do
            if action.type == "return" then
                start = guide.positionToOffset(state, action.start) + 1
                break
            end
        end

        local generated = [[
---@generic T
---@param value T
---@return Kinematic.Property<T>
---@overload fun(value:Kinematic.Vec2):Kinematic.Vec2Property
---@overload fun(value:Kinematic.Vec3):Kinematic.Vec3Property
---@overload fun(value:Kinematic.Color):Kinematic.ColorProperty
---@overload fun(value:Kinematic.Quaternion):Kinematic.QuaternionProperty
---@overload fun(value:Kinematic.Quad):Kinematic.QuadProperty
---@diagnostic disable-next-line: unused-local, unused-function, missing-return
local function _kinematic_property(value) end
]] .. table.concat(classes, "\n") .. "\n"
        for _, diff in ipairs(diffs) do
            if diff.start == start and diff.finish == start - 1 then
                diff.text = generated .. diff.text
                return diffs
            end
        end

        diffs[#diffs + 1] = { start = start, finish = start - 1, text = generated }
    end

    return diffs
end
