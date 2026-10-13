local parser = require "parser"
local guide = parser.guide
local files = require "files"
local paths = require "workspace.require-path"

local function field(table_node, name)
    if not table_node or table_node.type ~= "table" then return nil end
    for _, entry in ipairs(table_node) do
        local key = entry.type == "tablefield" and entry.field[1]
            or entry.type == "tableindex" and entry.index.type == "string" and entry.index[1]
        if key == name then return entry.value end
    end
end

local function resolve(node, visited)
    visited = visited or {}
    if not node or visited[node] then return nil end
    visited[node] = true
    if node.type == "getlocal" then return resolve(node.node.value, visited) end
    if node.type == "select" and node.sindex == 1 then return resolve(node.vararg, visited) end
    if node.type == "getfield" or node.type == "getindex" then
        local name = node.field and node.field[1] or node.index and node.index.type == "string" and node.index[1]
        local table_node = resolve(node.node, visited)
        if table_node and table_node.type == "table" then return resolve(field(table_node, name), visited) end
        return node
    end
    if node.type == "call" and node.node.type == "getglobal" and node.node[1] == "require"
        and node.args and node.args[1] and node.args[1].type == "string" and node.args[1][1] ~= "kinematic" then
        local uri = paths.findUrisByRequireName(guide.getUri(node), node.args[1][1])[1]
        local text = uri and files.getOriginText(uri)
        if not text or visited[uri] then return nil end
        visited[uri] = true
        local state = parser.compile(text, "Lua", "Lua 5.4")
        if not state or not state.ast then return nil end
        state.uri = uri
        state.ast.uri = uri
        for _, action in ipairs(state.ast) do
            if action.type == "return" then return resolve(action[1], visited) end
        end
        return nil
    end
    return node
end

local function api_call(call, name)
    if not call or call.type ~= "call" then return false end
    local callee = call.node
    if callee.type == "getlocal" then callee = resolve(callee) end
    if not callee or callee.type ~= "getfield" or callee.field[1] ~= name then return false end
    local api = resolve(callee.node)
    return api and api.type == "call" and api.node.type == "getglobal" and api.node[1] == "require"
        and api.args and api.args[1] and api.args[1][1] == "kinematic"
end

local function value_type(node)
    node = resolve(node)
    if not node then return "any" end
    if node.type == "integer" or node.type == "number" then return "number" end
    if node.type == "boolean" then return "boolean" end
    if node.type == "string" then return "string" end
    for _, name in ipairs { "vec2", "vec3", "color", "quat", "axis_angle", "quad" } do
        if api_call(node, name) then return "Kinematic.Value" end
    end
    -- shortcut: Computed defaults use any; add LuaLS inference if stricter value checks are needed.
    return "any"
end

local function class_name(call)
    local uri = guide.getUri(call)
    local hash = 0
    for index = 1, #uri do hash = (hash * 31 + uri:byte(index)) % 2147483647 end
    local parent = call.parent
    if parent and parent.type == "select" then parent = parent.parent end
    local name = call.start
    if parent and parent.type == "local" then
        local ordinal = 0
        guide.eachSourceType(guide.getRoot(call), "local", function(source)
            if source[1] == parent[1] and source.start <= parent.start then ordinal = ordinal + 1 end
        end)
        name = parent[1] .. "_" .. ordinal
    end
    return ("Kinematic.Custom_%d_%s"):format(hash, name)
end

local function property_type(schema)
    local choices = resolve(field(schema, "choices"))
    if choices and choices.type == "table" then
        local values = {}
        for _, entry in ipairs(choices) do
            local value = resolve(entry.value)
            if value and value.type == "string" then values[#values + 1] = ("%q"):format(value[1]) end
        end
        if #values > 0 then return table.concat(values, "|") end
    end
    return value_type(field(schema, "default"))
end

-- Describes component fields without executing project code.
function OnSetText(uri, text)
    if not uri:match("%.lua$") or uri:find("/kinematic/cache/luals/", 1, true)
        or not (text:find("object", 1, true) or text:find("component", 1, true)) then return nil end
    local state = parser.compile(text, "Lua", "Lua 5.4")
    if not state or not state.ast then return nil end
    state.uri = uri
    state.ast.uri = uri
    local diffs, classes = {}, {}
    guide.eachSourceType(state.ast, "call", function(call)
        local component = api_call(call, "component")
        if not component and not api_call(call, "object") then return end
        local options = call.args and resolve(call.args[1])
        if not options or options.type ~= "table" then return end
        local class = class_name(call)
        local parents = {}
        local definition = {}
        if component then
            parents[1] = "Kinematic.Component"
            for _, entry in ipairs(options) do
                local name = entry.type == "tablefield" and entry.field[1]
                    or entry.type == "tableindex" and entry.index.type == "string" and entry.index[1]
                local schema = resolve(entry.value)
                if name and field(schema, "default") then
                    definition[#definition + 1] = ("---@field [%q] Kinematic.Property<%s>"):format(name, property_type(schema))
                end
            end
        else
            local dimension = resolve(field(options, "dimension"))
            parents[1] = dimension and dimension[1] == 3 and "Kinematic.Script3D" or "Kinematic.Script2D"
            definition[#definition + 1] = ("---@field signal fun(self:%s, callback:fun(object:%s, frame:Kinematic.Frame)):Kinematic.Signal"):format(class, class)
            local components = resolve(field(options, "components"))
            if components and components.type == "table" then
                for _, item in ipairs(components) do
                    local source = resolve(item.value)
                    if api_call(source, "component") then parents[#parents + 1] = class_name(source) end
                end
            end
        end
        table.insert(definition, 1, ("\n---@class %s: %s"):format(class, table.concat(parents, ",")))
        classes[#classes + 1] = table.concat(definition, "\n")
        local finish = guide.positionToOffset(state, call.finish) + 1
        local start = guide.positionToOffset(state, call.start) + 1
        diffs[#diffs + 1] = { start = start, finish = start - 1, text = "(" }
        diffs[#diffs + 1] = { start = finish, finish = finish - 1, text = (") --[[@as %s]]"):format(class) }
    end)
    if #classes > 0 then
        diffs[#diffs + 1] = { start = #text + 1, finish = #text, text = table.concat(classes, "\n") .. "\n" }
    end
    return diffs
end
