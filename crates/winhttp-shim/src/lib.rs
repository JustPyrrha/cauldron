#![feature(decl_macro)]

use libc::c_void;
use once_cell::sync::OnceCell;
use paste::paste;
use std::{mem::transmute, path::PathBuf};
use windows_sys::{
    Win32::{
        Foundation::{MAX_PATH, SYSTEMTIME},
        Networking::WinHttp::{
            URL_COMPONENTS, WIN_HTTP_CREATE_URL_FLAGS, WINHTTP_ACCESS_TYPE,
            WINHTTP_AUTOPROXY_OPTIONS, WINHTTP_CURRENT_USER_IE_PROXY_CONFIG,
            WINHTTP_EXTENDED_HEADER, WINHTTP_HEADER_NAME, WINHTTP_OPEN_REQUEST_FLAGS,
            WINHTTP_PROXY_CHANGE_CALLBACK, WINHTTP_PROXY_INFO, WINHTTP_PROXY_RESULT,
            WINHTTP_PROXY_RESULT_EX, WINHTTP_PROXY_SETTINGS, WINHTTP_PROXY_SETTINGS_PARAM,
            WINHTTP_PROXY_SETTINGS_TYPE, WINHTTP_QUERY_CONNECTION_GROUP_RESULT,
            WINHTTP_STATUS_CALLBACK, WINHTTP_WEB_SOCKET_BUFFER_TYPE,
        },
        System::{
            LibraryLoader::{GetProcAddress, LoadLibraryA},
            SystemInformation::GetSystemDirectoryA,
            SystemServices::DLL_PROCESS_ATTACH,
        },
    },
    core::{GUID, PCWSTR, PWSTR, s},
};

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn DllMain(_: isize, reason: u32, _: usize) -> bool {
    if reason == DLL_PROCESS_ATTACH {
        unsafe {
            std::env::set_var("RUST_BACKTRACE", "full");
            LoadLibraryA(s!("cauldron\\cauldron.dll"));
        };
    }

    true
}

macro __lazy_export($(fn $f:ident($($i:ident: $a:ty),*) $(-> $r:ty)?);+;) {
    #[inline]
    #[must_use]
    pub fn __h_version() -> isize {
        static VERSION: OnceCell<isize> = OnceCell::new();
        *VERSION.get_or_init(|| unsafe {
            let mut buffer = [0u8; MAX_PATH as usize];
            let buffer_len = GetSystemDirectoryA(buffer.as_mut_ptr(), buffer.len() as u32);
            assert_ne!(buffer_len, 0u32);

            let dir = PathBuf::from(String::from_utf8(buffer[..buffer_len as usize].to_vec()).unwrap()).join("winhttp.dll");
            let dir = [dir.to_str().unwrap().as_bytes(), &[0u8]].concat();
            LoadLibraryA(dir.as_ptr()) as isize
        })
    }

    paste! {
        $(
            #[allow(clippy::many_single_char_names)]
            #[unsafe(export_name = "" $f "")]
            unsafe extern "system" fn [<__ $f:snake>]($($i: $a),*) $(-> $r)? {
                static [<$f:snake:upper>]: OnceCell<usize> = OnceCell::new();

                unsafe {
                    transmute::<usize, unsafe extern "system" fn($($a),*) $(-> $r)?>(
                        *[<$f:snake:upper>].get_or_init(|| {
                            GetProcAddress(
                                __h_version() as *mut c_void,
                                format!("{}\0", stringify!($f)).as_ptr(),
                            )
                            .unwrap() as usize
                        }),
                    )($($i),*)
                }
            }
        )*
    }
}

#[rustfmt::skip]
__lazy_export! {
    fn WinHttpAddRequestHeaders(hrequest : *mut c_void, lpszheaders : PCWSTR, dwheaderslength : u32, dwmodifiers : u32) -> bool;
    fn WinHttpAddRequestHeadersEx(hrequest : *mut c_void, dwmodifiers : u32, ullflags : u64, ullextra : u64, cheaders : u32, pheaders : *const WINHTTP_EXTENDED_HEADER) -> u32;
    fn WinHttpCheckPlatform() -> bool;
    fn WinHttpCloseHandle(hinternet : *mut c_void) -> bool;
    fn WinHttpConnect(hsession : *mut c_void, pswzservername : PCWSTR, nserverport : u16, dwreserved : u32) -> *mut c_void;
    fn WinHttpCrackUrl(pwszurl : PCWSTR, dwurllength : u32, dwflags : u32, lpurlcomponents : *mut URL_COMPONENTS) -> bool;
    fn WinHttpCreateProxyResolver(hsession : *const c_void, phresolver : *mut *mut c_void) -> u32;
    fn WinHttpCreateUrl(lpurlcomponents : *const URL_COMPONENTS, dwflags : WIN_HTTP_CREATE_URL_FLAGS, pwszurl : PWSTR, pdwurllength : *mut u32) -> bool;
    fn WinHttpDetectAutoProxyConfigUrl(dwautodetectflags : u32, ppwstrautoconfigurl : *mut PWSTR) -> bool;
    fn WinHttpFreeProxyResult(pproxyresult : *mut WINHTTP_PROXY_RESULT);
    fn WinHttpFreeProxyResultEx(pproxyresultex : *mut WINHTTP_PROXY_RESULT_EX);
    fn WinHttpFreeProxySettings(pwinhttpproxysettings : *const WINHTTP_PROXY_SETTINGS);
    fn WinHttpFreeProxySettingsEx(proxysettingstype : WINHTTP_PROXY_SETTINGS_TYPE, pproxysettingsex : *const c_void) -> u32;
    fn WinHttpFreeQueryConnectionGroupResult(presult : *mut WINHTTP_QUERY_CONNECTION_GROUP_RESULT);
    fn WinHttpGetDefaultProxyConfiguration(pproxyinfo : *mut WINHTTP_PROXY_INFO) -> bool;
    fn WinHttpGetIEProxyConfigForCurrentUser(pproxyconfig : *mut WINHTTP_CURRENT_USER_IE_PROXY_CONFIG) -> bool;
    fn WinHttpGetProxyForUrl(hsession : *mut c_void, lpcwszurl : PCWSTR, pautoproxyoptions : *mut WINHTTP_AUTOPROXY_OPTIONS, pproxyinfo : *mut WINHTTP_PROXY_INFO) -> bool;
    fn WinHttpGetProxyForUrlEx(hresolver : *const c_void, pcwszurl : PCWSTR, pautoproxyoptions : *const WINHTTP_AUTOPROXY_OPTIONS, pcontext : usize) -> u32;
    fn WinHttpGetProxyForUrlEx2(hresolver : *const c_void, pcwszurl : PCWSTR, pautoproxyoptions : *const WINHTTP_AUTOPROXY_OPTIONS, cbinterfaceselectioncontext : u32, pinterfaceselectioncontext : *const u8, pcontext : usize) -> u32;
    fn WinHttpGetProxyResult(hresolver : *const c_void, pproxyresult : *mut WINHTTP_PROXY_RESULT) -> u32;
    fn WinHttpGetProxyResultEx(hresolver : *const c_void, pproxyresultex : *mut WINHTTP_PROXY_RESULT_EX) -> u32;
    fn WinHttpGetProxySettingsEx(hresolver : *const c_void, proxysettingstype : WINHTTP_PROXY_SETTINGS_TYPE, pproxysettingsparam : *const WINHTTP_PROXY_SETTINGS_PARAM, pcontext : usize) -> u32;
    fn WinHttpGetProxySettingsResultEx(hresolver : *const c_void, pproxysettingsex : *mut c_void) -> u32;
    fn WinHttpGetProxySettingsVersion(hsession : *const c_void, pdwproxysettingsversion : *mut u32) -> u32;
    fn WinHttpOpen(pszagentw : PCWSTR, dwaccesstype : WINHTTP_ACCESS_TYPE, pszproxyw : PCWSTR, pszproxybypassw : PCWSTR, dwflags : u32) -> *mut c_void;
    fn WinHttpOpenRequest(hconnect : *mut c_void, pwszverb : PCWSTR, pwszobjectname : PCWSTR, pwszversion : PCWSTR, pwszreferrer : PCWSTR, ppwszaccepttypes : *const PCWSTR, dwflags : WINHTTP_OPEN_REQUEST_FLAGS) -> *mut c_void;
    fn WinHttpQueryAuthSchemes(hrequest : *mut c_void, lpdwsupportedschemes : *mut u32, lpdwfirstscheme : *mut u32, pdwauthtarget : *mut u32) -> bool;
    fn WinHttpQueryConnectionGroup(hinternet : *const c_void, pguidconnection : *const GUID, ullflags : u64, ppresult : *mut *mut WINHTTP_QUERY_CONNECTION_GROUP_RESULT) -> u32;
    fn WinHttpQueryDataAvailable(hrequest : *mut c_void, lpdwnumberofbytesavailable : *mut u32) -> bool;
    fn WinHttpQueryHeaders(hrequest : *mut c_void, dwinfolevel : u32, pwszname : PCWSTR, lpbuffer : *mut c_void, lpdwbufferlength : *mut u32, lpdwindex : *mut u32) -> bool;
    fn WinHttpQueryHeadersEx(hrequest : *const c_void, dwinfolevel : u32, ullflags : u64, uicodepage : u32, pdwindex : *mut u32, pheadername : *const WINHTTP_HEADER_NAME, pbuffer : *mut c_void, pdwbufferlength : *mut u32, ppheaders : *mut *mut WINHTTP_EXTENDED_HEADER, pdwheaderscount : *mut u32) -> u32;
    fn WinHttpQueryOption(hinternet : *mut c_void, dwoption : u32, lpbuffer : *mut c_void, lpdwbufferlength : *mut u32) -> bool;
    fn WinHttpReadData(hrequest : *mut c_void, lpbuffer : *mut c_void, dwnumberofbytestoread : u32, lpdwnumberofbytesread : *mut u32) -> bool;
    fn WinHttpReadDataEx(hrequest : *mut c_void, lpbuffer : *mut c_void, dwnumberofbytestoread : u32, lpdwnumberofbytesread : *mut u32, ullflags : u64, cbproperty : u32, pvproperty : *const c_void) -> u32;
    fn WinHttpReadProxySettings(hsession : *const c_void, pcwszconnectionname : PCWSTR, ffallbacktodefaultsettings : bool, fsetautodiscoverfordefaultsettings : bool, pdwsettingsversion : *mut u32, pfdefaultsettingsarereturned : *mut bool, pwinhttpproxysettings : *mut WINHTTP_PROXY_SETTINGS) -> u32;
    fn WinHttpReceiveResponse(hrequest : *mut c_void, lpreserved : *mut c_void) -> bool;
    fn WinHttpRegisterProxyChangeNotification(ullflags : u64, pfncallback : WINHTTP_PROXY_CHANGE_CALLBACK, pvcontext : *const c_void, hregistration : *mut *mut c_void) -> u32;
    fn WinHttpResetAutoProxy(hsession : *const c_void, dwflags : u32) -> u32;
    fn WinHttpSendRequest(hrequest : *mut c_void, lpszheaders : PCWSTR, dwheaderslength : u32, lpoptional : *const c_void, dwoptionallength : u32, dwtotallength : u32, dwcontext : usize) -> bool;
    fn WinHttpSetCredentials(hrequest : *mut c_void, authtargets : u32, authscheme : u32, pwszusername : PCWSTR, pwszpassword : PCWSTR, pauthparams : *mut c_void) -> bool;
    fn WinHttpSetDefaultProxyConfiguration(pproxyinfo : *mut WINHTTP_PROXY_INFO) -> bool;
    fn WinHttpSetOption(hinternet : *const c_void, dwoption : u32, lpbuffer : *const c_void, dwbufferlength : u32) -> bool;
    fn WinHttpSetProxySettingsPerUser(fproxysettingsperuser : bool) -> u32;
    fn WinHttpSetStatusCallback(hinternet : *mut c_void, lpfninternetcallback : WINHTTP_STATUS_CALLBACK, dwnotificationflags : u32, dwreserved : usize) -> WINHTTP_STATUS_CALLBACK;
    fn WinHttpSetTimeouts(hinternet : *mut c_void, nresolvetimeout : i32, nconnecttimeout : i32, nsendtimeout : i32, nreceivetimeout : i32) -> bool;
    fn WinHttpTimeFromSystemTime(pst : *const SYSTEMTIME, pwsztime : PWSTR) -> bool;
    fn WinHttpTimeToSystemTime(pwsztime : PCWSTR, pst : *mut SYSTEMTIME) -> bool;
    fn WinHttpUnregisterProxyChangeNotification(hregistration : *const c_void) -> u32;
    fn WinHttpWebSocketClose(hwebsocket : *const c_void, usstatus : u16, pvreason : *const c_void, dwreasonlength : u32) -> u32;
    fn WinHttpWebSocketCompleteUpgrade(hrequest : *const c_void, pcontext : usize) -> *mut c_void;
    fn WinHttpWebSocketQueryCloseStatus(hwebsocket : *const c_void, pusstatus : *mut u16, pvreason : *mut c_void, dwreasonlength : u32, pdwreasonlengthconsumed : *mut u32) -> u32;
    fn WinHttpWebSocketReceive(hwebsocket : *const c_void, pvbuffer : *mut c_void, dwbufferlength : u32, pdwbytesread : *mut u32, pebuffertype : *mut WINHTTP_WEB_SOCKET_BUFFER_TYPE) -> u32;
    fn WinHttpWebSocketSend(hwebsocket : *const c_void, ebuffertype : WINHTTP_WEB_SOCKET_BUFFER_TYPE, pvbuffer : *const c_void, dwbufferlength : u32) -> u32;
    fn WinHttpWebSocketShutdown(hwebsocket : *const c_void, usstatus : u16, pvreason : *const c_void, dwreasonlength : u32) -> u32;
    fn WinHttpWriteData(hrequest : *mut c_void, lpbuffer : *const c_void, dwnumberofbytestowrite : u32, lpdwnumberofbyteswritten : *mut u32) -> bool;
    fn WinHttpWriteProxySettings(hsession : *const c_void, fforceupdate : bool, pwinhttpproxysettings : *const WINHTTP_PROXY_SETTINGS) -> u32;
}
