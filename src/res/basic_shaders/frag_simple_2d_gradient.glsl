#version 330

out vec4 oColor;

// bottom left: (0, 0)
// top right:   (1, 1)
in vec2 iUv;

// the screen resolution in pixels
uniform vec2 uResolution;

// the mouse position measured in pixels from the bottom left
uniform vec2 uMouse;

// elapsed time in seconds
uniform float uTime;


vec2 screenToWorld(vec2 camPos, vec2 camSize, vec2 pos) {
    return camPos + (pos * 0.5 - 0.5) * camSize * 0.5;
}

void main() {
    vec2 camPos = vec2(0.0, 0.0);
    vec2 camSize = uResolution / max(uResolution.x, uResolution.y);

    vec2 p = screenToWorld(camPos, camSize, iUv);
    vec2 mouseP = screenToWorld(camPos, camSize, uMouse / uResolution);

    vec2 pos = 2.0 * (mouseP - p);

    oColor = vec4(vec2(1.0) - min(abs(pos), vec2(1.0)), (sin(uTime) + 1.0) * 0.25, 1.0);
}
