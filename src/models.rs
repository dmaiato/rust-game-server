
#[derive(Debug, Clone)] 
pub struct Player {
    pub addr: std::net::SocketAddr,
    pub name: String,
    pub score: i32,
}

#[derive(Debug, Clone)] 
pub struct Question {
    pub text: String,
    pub options: String,
    pub correct_option: char,
}