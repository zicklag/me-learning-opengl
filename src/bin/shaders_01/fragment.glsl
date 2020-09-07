#version 330 core
out vec4 FragColor;

in vec4 vertexColor;

uniform float time;

void main() {
    FragColor = vec4(sin(time * 3 + 0.5), sin(time * 3 + 1.3), sin(time * 3 + 2.5), 1.0);
}