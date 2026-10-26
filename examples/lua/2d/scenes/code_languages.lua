return function(s)
    local examples = {
        {language = "python", source = 'def hello(name):\n    return f"Hello, {name}!"\n\nprint(hello("Kinematic"))'},
        {language = "rust", source = 'fn main() {\n    let value: u32 = 42;\n    println!("{value}");\n}'},
        {language = "tsx", source = 'const Button = ({label}: {label: string}) => (\n    <button className="primary">{label}</button>\n);'},
        {language = "jsonc", source = '{\n    // Animation settings.\n    "fps": 60,\n    "resolution": [1280, 720],\n}'},
        {language = "sql", source = 'SELECT name, duration\nFROM scenes\nWHERE duration > 1\nORDER BY name;'},
        {language = "hlsl", source = 'float4 main() : SV_Target {\n    return float4(1.0, 0.5, 0.0, 1.0);\n}'},
        {language = "wgsl", source = '@fragment\nfn main() -> @location(0) vec4f {\n    return vec4f(1.0, 0.5, 0.0, 1.0);\n}'},
    }

    for _, example in ipairs(examples) do
        local code = s:code {
            id = "code_" .. example.language,
            name = example.language,
            language = example.language,
            code = example.source,
            size = 36,
            opacity = 0,
        }
        s:world_2d():add(code)
        code:to {opacity = 1}:play(1)
        s:wait(1)
        code:to {opacity = 0}:play(1)
        code:remove()
    end
end
