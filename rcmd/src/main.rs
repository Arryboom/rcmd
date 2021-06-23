fn main() {
	let args: Vec<String> = std::env::args().collect();

	if args.len() < 2 {
		println!("usage: {} <pid> <command ...>", &args[0]);
		return;
	}

	let pid = &args[1].parse::<u32>().unwrap();
	let cmd = &args[2..].join(" ");

	match rcmd(*pid, cmd) {
		Err(e) => println!("[-] {} (errno: {})", e, unsafe { winapi::um::errhandlingapi::GetLastError() }),
		Ok(()) => println!("[+] Completed with no known errors!")
	}
}

fn rcmd(pid: u32, cmd: &String) -> Result<(), &str> {
	
	println!("[*] Target PID: {}", pid);
	println!("[*] Command: {}", cmd);

	let h_proc = unsafe { winapi::um::processthreadsapi::OpenProcess(
				winapi::um::winnt::PROCESS_ALL_ACCESS,
				winapi::shared::minwindef::FALSE,
				pid)
	};

	if h_proc == winapi::shared::ntdef::NULL {
		return Err("Unable to get process handle");
	}

	let p_buffer = unsafe { winapi::um::memoryapi::VirtualAllocEx(
					h_proc,
					winapi::shared::ntdef::NULL,
					cmd.len(),
					winapi::um::winnt::MEM_COMMIT,
					winapi::um::winnt::PAGE_EXECUTE_READWRITE)					
	};

	if p_buffer == winapi::shared::ntdef::NULL {
		return Err("Unable to allocate remote memory");
	}

	let mut bytes_written: winapi::shared::basetsd::SIZE_T = 0;

	if unsafe { winapi::um::memoryapi::WriteProcessMemory(
			h_proc,
			p_buffer,
			cmd.as_ptr() as *const u8 as _,
			cmd.len(),
			&mut bytes_written)
	} == winapi::shared::minwindef::FALSE {
		return Err("Unable to write remote memory");
	}

	//let func: winapi::shared::ntdef::PVOID = winapi::um::winbase::WinExec as _; // TODO: raw deref points at some jmp table I think
	let h_k32 = unsafe { winapi::um::libloaderapi::GetModuleHandleA("kernel32\x00".as_ptr() as _) };
	let func: winapi::shared::ntdef::PVOID = unsafe { winapi::um::libloaderapi::GetProcAddress(h_k32, "WinExec\x00".as_ptr() as _) } as _;
	let func_opt: unsafe extern "system" fn(*mut winapi::ctypes::c_void) -> u32 = unsafe { std::mem::transmute(func) };

	println!("[*] Using data pointer at {:?}", p_buffer);
	println!("[*] Using function pointer at {:?}", func);

	let mut tid: winapi::shared::minwindef::DWORD = 0;

	let h_thread = unsafe { winapi::um::processthreadsapi::CreateRemoteThread(
												h_proc,
												winapi::shared::ntdef::NULL as _,
												0,
												Some(func_opt),
												p_buffer,
												0,
												&mut tid
	)};

	println!("[+] Spawned TID {} in PID {}", tid, pid);

	Ok(())
}
