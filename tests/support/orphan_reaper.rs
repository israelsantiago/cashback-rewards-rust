use std::collections::HashSet;
use std::process::Command;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref CONTAINERS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
    static ref INIT: std::sync::Once = std::sync::Once::new();
}

// Esta função C-like é chamada obrigatoriamente pelo Sistema Operacional 
// milissegundos antes do processo ser finalizado com sucesso.
extern "C" fn cleanup_on_exit() {
    reap_all();
}

pub fn track_container(id: &str) {
    INIT.call_once(|| {
        // Intercepta pânicos e cancelamentos manuais (Ctrl+C)
        let _ = ctrlc::set_handler(move || {
            reap_all();
            std::process::exit(1);
        });

        // Intercepta o fim da execução da suíte de testes com sucesso (Drop ignorado pelo static)
        unsafe {
            libc::atexit(cleanup_on_exit);
        }
    });

    if let Ok(mut set) = CONTAINERS.lock() {
        set.insert(id.to_string());
    }
}

pub fn reap_all() {
    if let Ok(set) = CONTAINERS.lock() {
        for id in set.iter() {
            let _ = Command::new("docker")
                .args(["rm", "-f", "-v", id])
                .output(); // Executa o expurgo físico via Docker Daemon
        }
    }
}
