extern crate gl;
extern crate nalgebra_glm;
use nalgebra_glm as glm;

use gl::types::{GLuint, GLint, GLenum, GLchar};
use std::ffi::{CString, CStr};

pub struct Program {
    id: GLuint
}

impl Program {
    pub fn from_shaders(shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl::CreateProgram() };
        for shader in shaders {
            unsafe { gl::AttachShader(program_id, shader.id()); }
        }
        unsafe { gl::LinkProgram(program_id); }
        let mut success: GLint = 1;
        unsafe {
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }
        if success == 0 {
            let mut len: GLint = 0;
            unsafe {
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }
            let error = create_whitespace_cstring_with_len(len as usize);
            unsafe {
                gl::GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut GLchar
                );
            }
            return Err(error.to_string_lossy().into_owned());
        }
        for shader in shaders {
            unsafe { gl::DetachShader(program_id, shader.id()); }
        }
        Ok(Program { id: program_id })
    }

    pub fn id(&self) -> GLuint { self.id }

    pub fn set_used(&self) {
        unsafe { gl::UseProgram(self.id); }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id); }
    }
}

pub struct Shader {
    id: GLuint
}

impl Shader {
    fn from_source(source: &CStr, kind: GLenum) -> Result<Shader, String> {
        let id = shader_from_source(source, kind)?;
        Ok(Shader { id })
    }
    pub fn from_vert_source(source: &CStr) -> Result<Shader, String> {
        Shader::from_source(source, gl::VERTEX_SHADER)
    }
    pub fn from_frag_source(source: &CStr) -> Result<Shader, String> {
        Shader::from_source(source, gl::FRAGMENT_SHADER)
    }
    pub fn from_geom_source(source: &CStr) -> Result<Shader, String> {
        Shader::from_source(source, gl::GEOMETRY_SHADER)
    }
    pub fn id(&self) -> GLuint {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id); }
    }
}

fn shader_from_source(source: &CStr, kind: GLuint) -> Result<GLuint, String> {
    let id: GLuint = unsafe { gl::CreateShader(kind) };
    let mut success: GLint = 1;
    let mut err_len: GLint = 0;
    unsafe {
        gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(id);
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut err_len);
    }
    if success == 0 {
        let error = create_whitespace_cstring_with_len(err_len as usize);
        unsafe {
            gl::GetShaderInfoLog(id, err_len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
        }
        return Err(error.to_string_lossy().into_owned());
    }
    Ok(id)
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    let mut buf: Vec<u8> = Vec::with_capacity(len as usize + 1);
    buf.extend([b' '].iter().cycle().take(len as usize));
    unsafe { CString::from_vec_unchecked(buf) }
}

pub struct GChar {
    cstr: CString
}

impl GChar {
    pub fn new(s: &str) -> GChar {
        GChar { cstr:  CString::new(s.as_bytes()).unwrap() }
    }
    pub fn ptr(&self) -> *const GLchar {
        self.cstr.as_ptr() as *const GLchar
    }
}

pub trait SendUniform {
    fn send_uniform(&self, prog_id: GLuint, name: &str) -> Result<(), String> {
        unsafe {
            let loc = get_uniform_location(prog_id, name)?;
            self.uniform(loc);
        }
        Ok(())
    }
    unsafe fn uniform(&self, loc: GLint);
}

pub trait SendUniforms {
    fn send_uniforms(&self, prog_id: GLuint) -> Result<(), String>;
}

unsafe fn get_uniform_location(prog_id: GLuint, name: &str) -> Result<i32, String>
{
    let loc = gl::GetUniformLocation(prog_id, GChar::new(name).ptr());
    if loc < 0 {
        return Err("Could not get uniform location".to_string());
    }
    return Ok(loc)
}

impl SendUniform for glm::Vec2 {
    unsafe fn uniform(&self, loc: GLint) {
        gl::Uniform2fv(loc, 1, self.as_ptr()); 
    }
}

impl SendUniform for glm::Vec3 {
    unsafe fn uniform(&self, loc: GLint) {
        gl::Uniform3fv(loc, 1, self.as_ptr()); 
    }
}

impl SendUniform for glm::Vec4 {
    unsafe fn uniform(&self, loc: GLint) {
        gl::Uniform4fv(loc, 1, self.as_ptr()); 
    }
}

impl SendUniform for glm::Mat3 {
    unsafe fn uniform(&self, loc: GLint) {
        gl::UniformMatrix3fv(loc, 1, gl::FALSE, self.as_ptr()); 
    }
}

impl SendUniform for glm::Mat4 {
    unsafe fn uniform(&self, loc: GLint) {
        gl::UniformMatrix4fv(loc, 1, gl::FALSE, self.as_ptr()); 
    }
}



