use serde::Deserialize;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::env;

#[derive(Debug, Deserialize)]
pub struct NiriWorkspace {
    pub id: u64,
    pub idx: u64,
    #[allow(dead_code)]
    pub name: Option<String>,
    #[allow(dead_code)]
    pub output: String,
    #[allow(dead_code)]
    pub is_urgent: bool,
    #[allow(dead_code)]
    pub is_active: bool,
    pub is_focused: bool,
    #[allow(dead_code)]
    pub active_window_id: Option<u64>,
}

pub struct NiriIpc {
    socket_path: String,
}

impl NiriIpc {
    pub fn new() -> Option<Self> {
        let socket_path = env::var("NIRI_SOCKET").ok()?;
        Some(Self { socket_path })
    }

    fn send_request(&self, request: &str) -> Result<String, std::io::Error> {
        let stream = UnixStream::connect(&self.socket_path)?;
        stream.set_read_timeout(Some(std::time::Duration::from_millis(500)))?;
        
        let mut stream_write = stream.try_clone()?;
        let mut reader = BufReader::new(stream);
        
        let json_request = format!("\"{}\"", request);
        writeln!(stream_write, "{}", json_request)?;
        stream_write.flush()?;
        
        let mut response = String::new();
        reader.read_line(&mut response)?;
        
        Ok(response)
    }

    pub fn get_workspaces(&self) -> Option<Vec<NiriWorkspace>> {
        let response = self.send_request("Workspaces").ok()?;
        
        #[derive(Deserialize)]
        struct WorkspacesData {
            #[serde(rename = "Workspaces")]
            workspaces: Vec<NiriWorkspace>,
        }
        
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "Ok")]
            ok: Option<WorkspacesData>,
        }
        
        serde_json::from_str::<Response>(&response)
            .ok()?
            .ok
            .map(|data| data.workspaces)
    }

    #[allow(dead_code)]
    pub fn get_focused_workspace(&self) -> Option<String> {
        let workspaces = self.get_workspaces()?;
        
        for ws in workspaces {
            if ws.is_focused {
                return Some(
                    ws.name
                        .unwrap_or_else(|| format!("{}", ws.id))
                );
            }
        }
        
        None
    }

    pub fn get_workspace_summary(&self) -> String {
        match self.get_workspaces() {
            Some(mut workspaces) => {
                if workspaces.is_empty() {
                    return String::from("Empty");
                }
                
                workspaces.sort_by_key(|w| w.idx);
                
                let occupied: Vec<String> = workspaces.iter()
                    .map(|ws| {
                        if ws.is_focused {
                            format!("[{}]", ws.idx)
                        } else {
                            format!("{}", ws.idx)
                        }
                    })
                    .collect();
                
                occupied.join(" ")
            }
            None => String::from("WS ?"),
        }
    }
}
