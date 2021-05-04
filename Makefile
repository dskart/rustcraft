glsls = $(wildcard assets/shaders/*.vert assets/shaders/*.frag)
spirvs = $(addsuffix .spv,$(glsls))

.PHONY: shaders
shaders: $(spirvs)

$(spirvs): %.spv: %
	glslangValidator -V $< -o $@

.PHONY: clean
clean:
	rm -f $(spirvs)