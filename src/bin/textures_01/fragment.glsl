#version 330 core
out vec4 FragColor;

in vec4 vertexColor;
in vec2 textureCoord;

uniform float time;
uniform sampler2D imageTexture;

void main() {
    FragColor = texture(imageTexture, textureCoord) * vertexColor;
}