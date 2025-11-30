mod models;
mod quiz_data;

use models::{Player, Question};
use quiz_data::load_and_shuffle_questions;
use quiz_data::reshuffle_questions;

use std::io::Error;
use std::net::UdpSocket;
use std::str;
use std::thread::sleep;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let questions_result: Result<Vec<Question>, Error> =
        load_and_shuffle_questions("resources/questions.txt");
    let mut questions: Vec<Question> = match questions_result {
        Ok(q) => q,
        Err(e) => {
            eprintln!("ERRO FATAL: Falha ao carregar perguntas: {}", e);
            eprintln!("Certifique-se de que 'questions.txt' está na pasta raiz.");
            return Err(e);
        }
    };

    if questions.is_empty() {
        eprintln!("ERRO FATAL: Nenhuma pergunta válida foi carregada. Verifique o arquivo.");
        return Ok(());
    }

    let socket: UdpSocket = UdpSocket::bind("0.0.0.0:10000")?;
    socket.set_read_timeout(Some(Duration::from_secs(600)))?;

    println!("\n=== SERVIDOR BATTLE QUIZ INICIADO NA PORTA 10000 (UDP) ===");
    println!(
        "{} perguntas carregadas. Aguardando 2 jogadores...",
        questions.len()
    );

    let mut buf: [u8; 2048] = [0; 2048];
    let mut players: Vec<Player> = Vec::new();

    // inicio (lobby)
    while players.len() < 2 {
        let (amt, src) = socket.recv_from(&mut buf)?;
        let msg: &str = str::from_utf8(&buf[..amt]).unwrap_or("").trim();

        if msg.starts_with("LOGIN:") {
            let name: String = msg.replace("LOGIN:", "");
            if !players.iter().any(|p: &Player| p.addr == src) {
                println!("Novo jogador conectado: {} [{}]", name, src);
                players.push(Player {
                    addr: src,
                    name: name.clone(),
                    score: 0,
                });

                let welcome: String = format!(
                    "BEM-VINDO! Aguardando oponente... (Jogadores: {}/2)",
                    players.len()
                );
                socket.send_to(welcome.as_bytes(), src)?;
            }
        }
    }

    println!("Dois jogadores conectados! Iniciando Jogo...");
    for p in &players {
        socket.send_to(b"JOGO_INICIADO", p.addr)?;
    }

    let mut question_index: usize = 0;
    let mut game_running: bool = true;
    let mut restart_loop: bool = false;

    // loop de jogo
    while game_running && question_index < questions.len() {
        let current_q: &Question = &questions[question_index];

        let q_msg: String = format!("PERGUNTA:{}\n{}", current_q.text, current_q.options);
        println!(
            "\nEnviando rodada {}: {}",
            question_index + 1,
            current_q.text
        );

        for p in &players {
            socket.send_to(q_msg.as_bytes(), p.addr)?;
        }

        let mut answered_count: i32 = 0;
        let mut first_error: bool = false;
        let mut round_finished: bool = false;
        let mut round_winner_idx: Option<usize> = None;
        let mut points_awarded: i32 = 0;

        // Loop de espera pelas respostas desta rodada
        while !round_finished && answered_count < 2 {
            // verificação do primeiro a responder
            let (amt, src) = socket.recv_from(&mut buf)?;
            let msg: &str = str::from_utf8(&buf[..amt]).unwrap_or("").trim();

            let player_idx: usize = if players[0].addr == src {
                0
            } else if players[1].addr == src {
                1
            } else {
                continue;
            };

            if msg.starts_with("RESPOSTA:") {
                let answer_char: char = msg.chars().last().unwrap_or(' ');
                println!(
                    "Jogador {} respondeu: {}",
                    players[player_idx].name, answer_char
                );

                if answered_count == 0 {
                    // primeiro que respondeu
                    if answer_char == current_q.correct_option {
                        players[player_idx].score += 5;
                        points_awarded = 5;
                        round_winner_idx = Some(player_idx);
                    } else {
                        first_error = true;
                    }
                } else {
                    // se o primeiro errar
                    if first_error {
                        if answer_char == current_q.correct_option {
                            players[player_idx].score += 3;
                            points_awarded = 3;
                            round_winner_idx = Some(player_idx);
                        } else {
                            // se ambos erram
                            points_awarded = 0;
                        }
                    }
                }
                answered_count += 1;
                if answered_count == 2 {
                    round_finished = true
                }
            }
        }

        // fim de rodada
        let placar_msg: String = format!(
            "PLACAR: {} = {} | {} = {}\nResultado da rodada: Vencedor: {} (+{})",
            players[0].name,
            players[0].score,
            players[1].name,
            players[1].score,
            match round_winner_idx {
                Some(idx) => &players[idx].name,
                None => "Ninguem",
            },
            points_awarded
        );

        println!("Fim da rodada. {}", placar_msg);

        for p in &players {
            socket.send_to(placar_msg.as_bytes(), p.addr)?;
        }

        // verifica se alguém atingiu 30 pontos
        for p in &players {
            if p.score >= 1 {
                // game_running = false;
                let win_msg: String =
                    format!("FIM DE JOGO! VENCEDOR: {} com {} pontos.", p.name, p.score);
                println!("{}", win_msg);
                for client in &players {
                    socket.send_to(win_msg.as_bytes(), client.addr)?;
                }
                restart_loop = true;
                break;
            }
        }

        // avança para a próxima pergunta
        question_index += 1;

        // se acabarem as perguntas
        if question_index >= questions.len() && game_running {
            println!("Acabaram todas as perguntas. O jogo encerra por falta de questões.");
            for p in &players {
                let msg: &str = "FIM DE JOGO: Todas as perguntas foram usadas.";
                socket.send_to(msg.as_bytes(), p.addr)?;
            }
            restart_loop = true;
        }

        if restart_loop {
            let restart_msg: &str = "DESEJA JOGAR NOVAMENTE? (S/N)";
            println!("ENVIANDO CONVITE DE REINICIO DE JOGO...");
            for client in &players {
                socket.send_to(restart_msg.as_bytes(), client.addr)?;
            }
            println!("AGUARDANDO RESPOSTA DOS JOGADORES...");

            let mut positive_answers: i32 = 0;

            while positive_answers < 2 {
                let (amt, _) = socket.recv_from(&mut buf)?;
                let msg: &str = str::from_utf8(&buf[..amt]).unwrap_or("").trim();

                if msg.starts_with("REINICIAR:") {
                    let answer_char: char = msg.chars().last().unwrap_or(' ');

                    if answer_char == 'S' {
                        positive_answers += 1;
                    } else if answer_char == 'N' {
                        restart_loop = false;
                        game_running = false;
                        break;
                    }
                }
            }

            if positive_answers >= 2 {
                question_index = 0;
                reshuffle_questions(&mut questions);
            }
        }

        sleep(Duration::from_secs(2));
    }

    Ok(())
}
