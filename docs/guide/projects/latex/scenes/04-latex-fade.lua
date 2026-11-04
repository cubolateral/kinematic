return function(s)
	local formula = s:latex_2d {
		text = [[E = mc^2]],
	}
	s:world_2d():add(formula)

	formula:fade([[x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}]]):play(1)
	s:wait(0.5)
	formula:fade([[P(A\mid B) = \frac{P(B\mid A)P(A)}{P(B)}]]):play(1)
	s:wait(0.5)
	formula:fade([[f(x) = \sum_{n=0}^{\infty} \frac{f^{(n)}(a)}{n!}(x-a)^n]]):play(1)
	s:wait(0.5)
	formula:fade([[\int_{-\infty}^{\infty} e^{-x^2}\,dx = \sqrt{\pi}]]):play(1)
	s:wait(0.5)
	formula:fade([[i\hbar\frac{\partial\Psi}{\partial t} = -\frac{\hbar^2}{2m}\nabla^2\Psi + V\Psi]]):play(1)

	s:wait(1)
end
