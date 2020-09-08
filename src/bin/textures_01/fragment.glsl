#version 330 core
out vec4 FragColor;

in vec4 vertexColor;
in vec2 textureCoord;

uniform float time;
uniform sampler2D imageTexture1;
uniform sampler2D imageTexture2;

void main() {
    FragColor = mix(texture(imageTexture1, textureCoord), texture(imageTexture2, textureCoord), 0.2) * vertexColor;
}