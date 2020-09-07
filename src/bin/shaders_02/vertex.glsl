# version  330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec4 aColor;

out vec4 vertexColor;

uniform float time;

void main() {
    vertexColor = vec4(aColor.x + sin(time) * 0.5, aColor.y + sin(time + 1) * 0.5, aColor.z, 1.0);
    gl_Position = vec4(aPos.x + sin(time) * 0.5, aPos.y + sin(time + 1) * 0.5, aPos.z, 1.0);
}