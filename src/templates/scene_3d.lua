return function(s)
    s:view("3d")

    local cube = s:cube {
        id = "cube",
        albedo = k.color("#60A5FA"),
    }
    s:world_3d():add(cube)

    cube:to { rotation = k.axis_angle(k.vec3(0, 1, 0), math.pi) }:play(1)

    s:wait(1)
end
