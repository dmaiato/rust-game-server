use std::net::UdpSocket;
use std::str;
use std::time::Duration;
use std::thread::sleep;

#[derive(Debug, Clone)]
struct Player {
    addr: std::net::SocketAddr,
    name: String,
    score: i32,
}

struct Question {
    text: String,
    options: String,
    correct_option: char,
}

fn main() -> std::io::Result<()> {

    let socket: UdpSocket = UdpSocket::bind("0.0.0.0:10000")?;

    // Timeout de leitura
    socket.set_read_timeout(Some(Duration::from_secs(600)))?;

    println!("=== SERVIDOR BATTLE QUIZ INICIADO NA PORTA 10000 (UDP) ===");
    println!("Aguardando 2 jogadores...");

    let mut buf: [u8; 2048] = [0; 2048]; // Buffer para receber dados
    let mut players: Vec<Player> = Vec::new();

    // === FASE 1: LOBBY (Aguardar Conexões) ===
    while players.len() < 2 {
        let (amt, src) = socket.recv_from(&mut buf)?;
        let msg: &str = str::from_utf8(&buf[..amt]).unwrap_or("").trim();

        // "LOGIN:NomeDoJogador"
        if msg.starts_with("LOGIN:") {
            let name: String = msg.replace("LOGIN:", "");

            // Verifica se o jogador já existe (pelo IP/Porta)
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

    // Envia mensagem de início para ambos
    for p in &players {
        socket.send_to(b"JOGO_INICIADO", p.addr)?;
    }

    // Banco de Perguntas
    let questions: Vec<Question> = vec![
        Question {
            text: String::from("Qual protocolo nao garante entrega?"),
            options: String::from("A) TCP | B) UDP | C) HTTP"),
            correct_option: 'B',
        },
        Question {
            text: String::from("Qual a porta padrao do HTTP?"),
            options: String::from("A) 80 | B) 21 | C) 443"),
            correct_option: 'A',
        },
        Question {
            text: String::from("O que significa IP?"),
            options: String::from("A) Internet Protocol | B) Intranet Port | C) Internal Process"),
            correct_option: 'A',
        },
        Question {
            text: String::from("Camada 4 do modelo OSI?"),
            options: String::from("A) Rede | B) Transporte | C) Sessao"),
            correct_option: 'B',
        },
        Question {
            text: String::from("Rust eh uma linguagem de...?"),
            options: String::from("A) Script | B) Markup | C) Sistema"),
            correct_option: 'C',
        },
    ];

    let mut question_index: usize = 0;
    let mut game_running: bool = true;

    // === FASE 2: LOOP DO JOGO ===
    while game_running && question_index < questions.len() {

        let current_q: &Question = &questions[question_index];

        // Formata a mensagem da pergunta
        let q_msg: String = format!("PERGUNTA:{}\n{}", current_q.text, current_q.options);
        println!(
            "\nEnviando rodada {}: {}",
            question_index + 1,
            current_q.text
        );

        // Envia para ambos os jogadores
        for p in &players {
            socket.send_to(q_msg.as_bytes(), p.addr)?;
        }

        // Lógica da Rodada
        let mut answered_count: i32 = 0;
        let mut first_error: bool = false;
        let mut round_finished: bool = false;
        let mut round_winner_idx: Option<usize> = None; // Indice do jogador no vetor
        let mut points_awarded: i32 = 0;

        // Loop de espera pelas respostas desta rodada
        while !round_finished && answered_count < 2 {
            let (amt, src) = socket.recv_from(&mut buf)?;
            let msg: &str = str::from_utf8(&buf[..amt]).unwrap_or("").trim();

            // Identifica qual jogador enviou (0 ou 1)
            let player_idx: usize = if players[0].addr == src {
                0
            } else if players[1].addr == src {
                1
            } else {
                continue;
            };

            let opponent_idx: usize = if player_idx == 0 { 1 } else { 0 };

            if msg.starts_with("RESPOSTA:") {
                let answer_char = msg.chars().last().unwrap_or(' ');
                println!(
                    "Jogador {} respondeu: {}",
                    players[player_idx].name, answer_char
                );

                if answered_count == 0 {
                    // === PRIMEIRO A RESPONDER ===
                    if answer_char == current_q.correct_option {
                        // Acertou de primeira: +5 pontos
                        players[player_idx].score += 5;
                        points_awarded = 5;
                        round_winner_idx = Some(player_idx);
                        round_finished = true;
                    } else {
                        // Errou: Passa a vez
                        first_error = true;
                        let aviso_erro: &str = "VOCE ERROU! Aguardando adversario...";
                        socket.send_to(aviso_erro.as_bytes(), players[player_idx].addr)?;

                        let aviso_oponente = "O ADVERSARIO ERROU! Sua vez de tentar (+3 pts)...";
                        socket.send_to(aviso_oponente.as_bytes(), players[opponent_idx].addr)?;
                    }
                } else {
                    // === SEGUNDO A RESPONDER (Se o primeiro errou) ===
                    if first_error {
                        if answer_char == current_q.correct_option {
                            // Adversário acertou: +3 pontos
                            players[player_idx].score += 3;
                            points_awarded = 3;
                            round_winner_idx = Some(player_idx);
                        } else {
                            // Ambos erraram: 0 pontos
                            points_awarded = 0;
                        }
                        round_finished = true;
                    }
                }
                answered_count += 1;
            }
        }

        // === FIM DA RODADA ===
        let placar_msg = format!(
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

        // Verifica Condição de Vitória (30 pontos)
        for p in &players {
            if p.score >= 30 {
                game_running = false;
                let win_msg = format!("FIM DE JOGO! VENCEDOR: {}", p.name);
                println!("{}", win_msg);
                // Avisa ambos
                for client in &players {
                    socket.send_to(win_msg.as_bytes(), client.addr)?;
                }
            }
        }

        // Avança pergunta ou reinicia se acabarem as perguntas (mas não atingiu 30 pts)
        question_index += 1;
        if question_index >= questions.len() && game_running {
            println!("Acabaram as perguntas do banco! Reiniciando ciclo...");
            question_index = 0; // Loop simples para não travar o jogo
        }

        // Pequena pausa antes da próxima questão
        sleep(Duration::from_secs(2));
    }

    Ok(())
}
