OBJECTS := $(shell find ./ -mindepth 2 -name 'Dockerfile')

all: $(OBJECTS)

%/Dockerfile: Dockerfile.j2 render_template.py
	python3 render_template.py "$<" "{\"target_file\":\"$@\"}" > "$@"
