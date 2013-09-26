#[link(name = "sndfile",
       vers = "0.0.1",
       author = "letang.jeremy@gmail.com",
       uuid = "F8CC5AA5-95DC-449C-B7DD-A6663DAC8E51",
       url = "http://https://github.com/JeremyLetang/rust-sndfile")];

#[desc = "Libsndfile binding for sfml"];
#[license = "GPL/LGPL"];
#[crate_type = "lib"];

extern mod extra;

use std::{str, ptr, vec};

#[doc(hidden)]
#[cfg(target_os="macos")]
#[cfg(target_os="linux")]
#[cfg(target_os="win32")]
mod libsndfile {
    #[link_args="-lsndfile"]
    extern {}
}

#[doc(hidden)]
mod ffi;

/// The SndInfo structure is for passing data between the calling 
/// function and the library when opening a file for reading or writing.
#[deriving(Clone)]
pub struct SndInfo {
    frames : i64,
    samplerate : i32,
    channels : i32,
    format : i32,
    sections : i32,
    seekable : i32
}

pub struct FormatInfo
{   format : i32,
    name : ~str,
    extension : ~str
}

/// Modes availables for the open function.
///
/// * Read - Read only mode
/// * Write - Write only mode
/// * ReadWrite - Read and Write mode
pub enum OpenMode {
    Read    = ffi::SFM_READ as i32,
    Write   = ffi::SFM_WRITE as i32,
    ReadWrite    = ffi::SFM_RDWR as i32
}

/// Type of strings available for method get_string()
pub enum StringSoundType {
    Title       = ffi::SF_STR_TITLE as i32,
    Copyright   = ffi::SF_STR_COPYRIGHT as i32,
    Software    = ffi::SF_STR_SOFTWARE as i32,
    Artist      = ffi::SF_STR_ARTIST as i32,
    Comment     = ffi::SF_STR_COMMENT as i32,
    Date        = ffi::SF_STR_DATE as i32, 
    Album       = ffi::SF_STR_ALBUM as i32,
    License     = ffi::SF_STR_LICENSE as i32,
    TrackNumber = ffi::SF_STR_TRACKNUMBER as i32,
    Genre       = ffi::SF_STR_GENRE as i32
}

/// Types of error who can be return by API functions
pub enum Error {
    NoError             = ffi::SF_ERR_NO_ERROR as i32,
    UnrecognizedFormat  = ffi::SF_ERR_UNRECOGNISED_FORMAT as i32,
    SystemError         = ffi::SF_ERR_SYSTEM as i32,
    MalformedFile       = ffi::SF_ERR_MALFORMED_FILE as i32,
    UnsupportedEncoding = ffi::SF_ERR_UNSUPPORTED_ENCODING as i32,
}


/// Enum to set the offset with method seek
///
/// * SeekSet - The offset is set to the start of the audio data plus offset (multichannel) frames.
/// * SeekCur - The offset is set to its current location plus offset (multichannel) frames.
/// * SeekEnd - The offset is set to the end of the data plus offset (multichannel) frames.
pub enum SeekMode {
    SeekSet = ffi::SEEK_SET as i32, 
    SeekCur = ffi::SEEK_CUR as i32, 
    SeekEnd = ffi::SEEK_END as i32  
}

pub struct SndFile {
    priv handle : *ffi::SNDFILE,
    priv info : ~SndInfo
}

impl SndFile {
    #[fixed_stack_segment] #[inline(never)]
    pub fn new(path : ~str, mode : OpenMode) -> Result<SndFile, ~str> {
        let info : ~SndInfo = ~SndInfo {frames : 0, samplerate : 0, channels : 0, format : 0, sections : 0, seekable : 0};
        let tmp_sndfile = unsafe { ffi::sf_open(path.to_c_str().unwrap(), mode as i32, &*info) };
        if tmp_sndfile.is_null() {
            Err(unsafe { str::raw::from_c_str(ffi::sf_strerror(ptr::null())) })
        } else {
            Ok(SndFile {
                handle :    tmp_sndfile,
                info :      info
            })
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn new_with_fd(fd : i32, mode : OpenMode, close_desc : bool) -> Result<SndFile, ~str> {
        let info : ~SndInfo = ~SndInfo {frames : 0, samplerate : 0, channels : 0, format : 0, sections : 0, seekable : 0};
        let tmp_sndfile = match close_desc {
            true    => unsafe { ffi::sf_open_fd(fd, mode as i32, &*info, ffi::SF_TRUE) },
            false   => unsafe { ffi::sf_open_fd(fd, mode as i32, &*info, ffi::SF_FALSE) }
        };
        if tmp_sndfile.is_null() {
            Err(unsafe { str::raw::from_c_str(ffi::sf_strerror(ptr::null())) })
        } else {
            Ok(SndFile {
                handle :    tmp_sndfile,
                info :      info
            })
        }
    }

    pub fn get_sndinfo(&self) -> ~SndInfo {
        self.info.clone()
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn get_string(&self, string_type : StringSoundType) -> Option<~str> {
        let c_string = unsafe {
            ffi::sf_get_string(self.handle, string_type as i32)
        };
        if c_string.is_null() {
            None
        } else {
            Some(unsafe { str::raw::from_c_str(c_string) })
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn set_string(&mut self, string_type : StringSoundType, string : ~str) -> Error {
        unsafe {
            ffi::sf_set_string(self.handle, string_type as i32, string.to_c_str().unwrap())
        }
    }    

    #[fixed_stack_segment] #[inline(never)]
    pub fn check_format<'r>(info : &'r SndInfo) -> bool {
        match unsafe {ffi::sf_format_check(info) } {
            ffi::SF_TRUE    => true,
            ffi::SF_FALSE   => false,
            _               => unreachable!()
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn close(&self) -> Error {
        unsafe {
            ffi::sf_close(self.handle)
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn write_sync(&mut self) -> () {
        unsafe {
            ffi::sf_write_sync(self.handle)
        }
    }   

    #[fixed_stack_segment] #[inline(never)]
    pub fn seek(&mut self, frames : i64, whence : SeekMode) -> i64{
        unsafe {
            ffi::sf_seek(self.handle, frames, whence as i32)
        }
    }    

    // READ FUNCTIONS

    #[fixed_stack_segment] #[inline(never)]
    pub fn read_i16<'r>(&'r mut self, array : &'r mut [i16], items : i64) -> i64 {
        unsafe {
            ffi::sf_read_short(self.handle, vec::raw::to_mut_ptr::<i16>(array), items)
        }        
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn read_i32<'r>(&'r mut self, array : &'r mut [i32], items : i64) -> i64 {
        unsafe {
            ffi::sf_read_int(self.handle, vec::raw::to_mut_ptr::<i32>(array), items)
        }        
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn read_f32<'r>(&'r mut self, array : &'r mut [f32], items : i64) -> i64 {
        unsafe {
            ffi::sf_read_float(self.handle, vec::raw::to_mut_ptr::<f32>(array), items)
        }        
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn read_f64<'r>(&'r mut self, array : &'r mut [f64], items : i64) -> i64 {
        unsafe {
            ffi::sf_read_double(self.handle, vec::raw::to_mut_ptr::<f64>(array), items)
        }        
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn readf_i16<'r>(&'r mut self, array : &'r mut [i16], frames : i64) -> i64 {
        unsafe {
            ffi::sf_readf_short(self.handle, vec::raw::to_mut_ptr::<i16>(array), frames)
        }        
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn readf_i32<'r>(&'r mut self, array : &'r mut [i32], frames : i64) -> i64 {
        unsafe {
            ffi::sf_readf_int(self.handle, vec::raw::to_mut_ptr::<i32>(array), frames)
        }        
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn readf_f32<'r>(&'r mut self, array : &'r mut [f32], frames : i64) -> i64 {
        unsafe {
            ffi::sf_readf_float(self.handle, vec::raw::to_mut_ptr::<f32>(array), frames)
        }        
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn readf_f64<'r>(&'r mut self, array : &'r mut [f64], frames : i64) -> i64 {
        unsafe {
            ffi::sf_readf_double(self.handle, vec::raw::to_mut_ptr::<f64>(array), frames)
        }        
    }

    // WRITE FUNCTIONS

    #[fixed_stack_segment] #[inline(never)]
    pub fn write_i16<'r>(&'r mut self, array : &'r mut [i16], items : i64) -> i64 {
        unsafe {
            ffi::sf_write_short(self.handle, vec::raw::to_mut_ptr::<i16>(array), items)
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn write_i32<'r>(&'r mut self, array : &'r mut [i32], items : i64) -> i64 {
        unsafe {
            ffi::sf_write_int(self.handle, vec::raw::to_mut_ptr::<i32>(array), items)
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn write_f32<'r>(&'r mut self, array : &'r mut [f32], items : i64) -> i64 {
        unsafe {
            ffi::sf_write_float(self.handle, vec::raw::to_mut_ptr::<f32>(array), items)
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn write_f64<'r>(&'r mut self, array : &'r mut [f64], items : i64) -> i64 {
        unsafe {
            ffi::sf_write_double(self.handle, vec::raw::to_mut_ptr::<f64>(array), items)
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn writef_i16<'r>(&'r mut self, array : &'r mut [i16], frames : i64) -> i64 {
        unsafe {
            ffi::sf_writef_short(self.handle, vec::raw::to_mut_ptr::<i16>(array), frames)
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn writef_i32<'r>(&'r mut self, array : &'r mut [i32], frames : i64) -> i64 {
        unsafe {
            ffi::sf_writef_int(self.handle, vec::raw::to_mut_ptr::<i32>(array), frames)
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn writef_f32<'r>(&'r mut self, array : &'r mut [f32], frames : i64) -> i64 {
        unsafe {
            ffi::sf_writef_float(self.handle, vec::raw::to_mut_ptr::<f32>(array), frames)
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn writef_f64<'r>(&'r mut self, array : &'r mut [f64], frames : i64) -> i64 {
        unsafe {
            ffi::sf_writef_double(self.handle, vec::raw::to_mut_ptr::<f64>(array), frames)
        }
    }

    // Error handlers

    #[fixed_stack_segment] #[inline(never)]
    pub fn error(&self) -> Error {
        unsafe {
            ffi::sf_error(self.handle)
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn string_error(&self) -> ~str {
        unsafe {
            str::raw::from_c_str(ffi::sf_strerror(self.handle))
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn error_number(error_num : Error) -> ~str {
        unsafe {
            str::raw::from_c_str(ffi::sf_error_number(error_num as i32))
        }
    }

}

