
pub const PLANAR_FRAGMENT_SHADER: &'static str = "#version 330 core

#ifdef GL_ES
// Set default precision to medium
precision mediump int;
precision mediump float;
#endif

uniform sampler2D tex_y;
uniform sampler2D tex_u;
uniform sampler2D tex_v;
uniform int tex_format;
uniform float alpha;
uniform float tex_offset;
uniform float imageWidth;
uniform float imageHeight;
uniform bool enableHDR;
uniform bool enableGaussianBlur;


in vec2 TexCoord;
out vec4 FragColor;

float gamma = 2.2;
vec3 toLinear(in vec3 colour) { return pow(colour, vec3(gamma)); }
vec3 toHDR(in vec3 colour, in float range) { return toLinear(colour) * range; }
const float M_PI = 3.1415926535897932384626433832795;
vec4 GaussianBlur(sampler2D tex0, vec2 texCoordinates, float blurAmnt, int passingTurn, float sigma, float numBlurPixelsPerSide)
{
    vec4 outputColor;
    vec2 blurMultiplyVec;
    if (passingTurn == 0) blurMultiplyVec = vec2(1.0, 0.0);
    else blurMultiplyVec = vec2(0.0, 1.0);

    // Incremental Gaussian Coefficent Calculation (See GPU Gems 3 pp. 877 - 889)
    vec3 incrementalGaussian;
    incrementalGaussian.x = 1.0 / (sqrt(2.0 * M_PI) * sigma);
    incrementalGaussian.y = exp(-0.5f / (sigma * sigma));
    incrementalGaussian.z = incrementalGaussian.y * incrementalGaussian.y;

    vec4 avgValue = vec4(0.0, 0.0, 0.0, 0.0);
    float coefficientSum = 0.0;

    // Take the central sample first...
    avgValue += texture(tex0, texCoordinates ) * incrementalGaussian.x;
    coefficientSum += incrementalGaussian.x;
    incrementalGaussian.xy *= incrementalGaussian.yz;

    // Go through the remaining 8 vertical samples (4 on each side of the center)
    for (float i = 1.0; i <= numBlurPixelsPerSide; i++)
    {
        avgValue += texture(tex0, texCoordinates  - i * blurAmnt * blurMultiplyVec) * incrementalGaussian.x;
        avgValue += texture(tex0, texCoordinates  + i * blurAmnt * blurMultiplyVec) * incrementalGaussian.x;
        coefficientSum += 2.0 * incrementalGaussian.x;
        incrementalGaussian.xy *= incrementalGaussian.yz;
    }

    outputColor = avgValue / coefficientSum;

    return outputColor;
}
void main()
{
    //if(TexCoord.x > 1.0 - tex_offset){
    //    FragColor.a = 0;
    //    FragColor.r = 0;
    //    FragColor.g = 0;
    //    FragColor.b = 0;
    //return;
    //}
    vec3 yuv;
    vec4 rgba;
    if(tex_format == 0 || tex_format == 1){
        if(tex_format == 0){
            yuv.r = texture(tex_y, TexCoord).r - 0.0625;
        }else{
            yuv.r = texture(tex_y, TexCoord).r;
        }
        yuv.g = texture(tex_u, TexCoord).r - 0.5;
        yuv.b = texture(tex_v, TexCoord).r - 0.5;
    }else if(tex_format == 2){ // rgb
        yuv = texture(tex_y, TexCoord).rgb;
    }else if(tex_format == 3){ // gray8
        yuv.r = texture(tex_y, TexCoord).r;
    }else if(tex_format == 6){ //BGR
        yuv = texture(tex_y, TexCoord).bgr;
    }else if(tex_format == 10){//yuv420p10le yuv444p10le
        vec3 yuv_l;
        vec3 yuv_h;
        yuv_l.x = texture(tex_y, TexCoord).r;
        yuv_h.x = texture(tex_y, TexCoord).a;
        yuv_l.y = texture(tex_u, TexCoord).r;
        yuv_h.y = texture(tex_u, TexCoord).a;
        yuv_l.z = texture(tex_v, TexCoord).r;
        yuv_h.z = texture(tex_v, TexCoord).a;
        yuv = (yuv_l * 255.0 + yuv_h * 255.0 * 256.0) / (1023.0) - vec3(16.0 / 255.0, 0.5, 0.5);
    }else if(tex_format == 8 || tex_format == 9){ //NV12 | NV21
        yuv.r = texture(tex_y, TexCoord).r - 0.0625;
        vec4 uv = texture( tex_u, TexCoord);
        //TODO: check if the modifications I made are working. I exchanged a for g
        if(tex_format == 9){ //NV21
            yuv.g = uv.g - 0.5;
            yuv.b = uv.r - 0.5;
        }else{ //NV12
            yuv.g = uv.r - 0.5;
            yuv.b = uv.g - 0.5;
        }
    }else if(tex_format == 16 || tex_format == 17){ //YUV16 YUVJ16
        if(tex_format == 16){
            yuv.r = texture(tex_y, TexCoord).r - 0.0625;
        }else{
            yuv.r = texture(tex_y, TexCoord).r;
        }
        yuv.g = texture(tex_u, TexCoord).r - 0.5;
        yuv.b = texture(tex_v, TexCoord).r - 0.5;
    }

    if(tex_format == 0 || tex_format == 10 || tex_format == 16){//yuv | p10le | //YUV16
        rgba.r = yuv.r + 1.596 * yuv.b;
        rgba.g = yuv.r - 0.813 * yuv.b - 0.391 * yuv.g;
        rgba.b = yuv.r + 2.018 * yuv.g;
    }else if(tex_format == 1 || tex_format == 17){ //yuy-jpeg || YUVJ16
        rgba.r = yuv.r + 1.402 * yuv.b;
        rgba.g = yuv.r - 0.34413 * yuv.g - 0.71414 * yuv.b;
        rgba.b = yuv.r + 1.772 * yuv.g;
        //vec3 rgb_;
        //rgb_ = mat3(1.0, 1.0, 1.0,
		//0.0, -0.39465, 2.03211,
		//1.13983, -0.58060, 0.0) * yuv;
        //rgba = vec4(rgb_,alpha);
    }
    else if(tex_format == 2){ //rgb
        rgba.rgb = yuv.rgb;
    }else if(tex_format == 3){ //gray8
        rgba.r = yuv.r;
        rgba.g = yuv.r;
        rgba.b = yuv.r;
    }else if(tex_format == 6){ //BGR
        rgba.r = yuv.b;
        rgba.g = yuv.g;
        rgba.b = yuv.r;
    }else if(tex_format == 19){ // BGGR
        vec2 firstRed = vec2(1,1);
        rgba.r = texture(tex_y, TexCoord).r;
        rgba.g = texture(tex_u, TexCoord).r;
        rgba.b = texture(tex_v, TexCoord).r;
        //        //        BGGR = 19,
        //        //        RGGB = 20 ,
        //        //        GRBG = 21 ,
        //        //        GBRG = 22 ,
    }else if(tex_format == 20){ //RGGB
        vec2 firstRed = vec2(0,0);
    }else if(tex_format == 21){ //GRBG
        vec2 firstRed = vec2(0,1);
    }else if(tex_format == 22){ //GBRG
        vec2 firstRed = vec2(1,0);
    }else if(tex_format == 23){//BGR565
        rgba.rgb = texture(tex_y, TexCoord).bgr;
    }else{ //其它
        rgba.r = yuv.r + 1.596 * yuv.b;
        rgba.g = yuv.r - 0.813 * yuv.b - 0.391 * yuv.g;
        rgba.b = yuv.r + 2.018 * yuv.g;
    }
    rgba.a = alpha;
    if(enableHDR){
        rgba.rgb = toHDR(rgba.rgb,1.0);
    }
    FragColor = rgba;
    //FragColor = vec4(0.5, 0.0, 0.0, 1.0);
//    sampler2D tex0, vec2 texCoordinates, float blurAmnt, int passingTurn, float sigma, float numBlurPixelsPerSide
    //if(enableGaussianBlur)//        
    //{
    //    gl_FragColor *= GaussianBlur(tex_y, TexCoord, 0, 1, 1, 9);
    //}
    //else{
//        gl_FragColor.a = 0.5;
    //}
}";