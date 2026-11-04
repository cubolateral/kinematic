return function(s)
	local code = s:code {
		code = [=[local function fibonacci(n)
    if n < 2 then return n end
    return fibonacci(n - 1) + fibonacci(n - 2)
end

print(fibonacci(10)) -- Result: 55.]=],
		language = "lua",
		size = 24,
	}
	s:world_2d():add(code)

	code:fade([=[local function factorial(n)
    if n < 2 then return 1 end
    return n * factorial(n - 1)
end

print(factorial(5)) -- Result: 120.]=]):play(1)
	s:wait(0.5)
	code:fade([=[local function gcd(a, b)
    while b ~= 0 do
        a, b = b, a % b
    end

    return a
end

print(gcd(48, 18)) -- Result: 6.]=]):play(1)
	s:wait(0.5)
	code:fade([=[local function is_prime(n)
    if n < 2 then return false end

    for divisor = 2, math.floor(math.sqrt(n)) do
        if n % divisor == 0 then return false end
    end

    return true
end

print(is_prime(29)) -- Result: true.]=]):play(1)
	s:wait(0.5)

	s:wait(1)
end
