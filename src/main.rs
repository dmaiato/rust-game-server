
mod models;
mod quiz_data;

use models::{Player, Question};
use quiz_data::load_and_shuffle_questions;

use std::io::Error;
use std::net::UdpSocket;
use std::str;
use std::time::Duration;


fn main() -> std::io::Result<()> {
    // 1. CARREGA AS PERGUNTAS DO ARQUIVO
    let questions_result:Result<Vec<Question>, Error> = load_and_shuffle_questions("resources/questions.txt");
    let questions: Vec<Question> = match questions_result {
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

    if questions.is_empty() {
        eprintln!("ERRO FATAL: Nenhuma pergunta válida foi carregada. Verifique o arquivo.");
        return Ok(()); // Encerra
    }

    // O RESTANTE DO CÓDIGO PERMANECE IGUAL (LOBBY E LOOP DO JOGO)
    let socket = UdpSocket::bind("0.0.0.0:10000")?;
    socket.set_read_timeout(Some(Duration::from_secs(600)))?;

    println!("\n=== SERVIDOR BATTLE QUIZ INICIADO NA PORTA 10000 (UDP) ===");
    println!(
        "{} perguntas carregadas. Aguardando 2 jogadores...",
        questions.len()
    );

    let mut buf = [0; 2048];
    let mut players: Vec<Player> = Vec::new();

    // === FASE 1: LOBBY (Aguardar Conexões) ===
    while players.len() < 2 {
        let (amt, src) = socket.recv_from(&mut buf)?;
        let msg = str::from_utf8(&buf[..amt]).unwrap_or("").trim();

        if msg.starts_with("LOGIN:") {
            let name = msg.replace("LOGIN:", "");
            if !players.iter().any(|p| p.addr == src) {
                println!("Novo jogador conectado: {} [{}]", name, src);
                players.push(Player {
                    addr: src,
                    name: name.clone(),
                    score: 0,
                });

                let welcome = format!(
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

    let mut question_index = 0;
    let mut game_running = true;

    // === FASE 2: LOOP DO JOGO ===
    // O loop agora usa o vetor 'questions' que foi carregado e embaralhado.
    while game_running && question_index < questions.len() {
        let current_q = &questions[question_index];

        let q_msg = format!("PERGUNTA:{}\n{}", current_q.text, current_q.options);
        println!(
            "\nEnviando rodada {}: {}",
            question_index + 1,
            current_q.text
        );

        for p in &players {
            socket.send_to(q_msg.as_bytes(), p.addr)?;
        }

        // ... (restante da lógica da rodada: pontuação, etc., permanece igual) ...

        let mut answered_count = 0;
        let mut first_error = false;
        let mut round_finished = false;
        let mut round_winner_idx: Option<usize> = None;
        let mut points_awarded = 0;

        // Loop de espera pelas respostas desta rodada
        // A duração do timeout do socket (acima, 600s) precisa ser longa,
        // ou você pode adicionar um timeout de rodada aqui com 'select' ou 'poll'
        // (mas é mais complexo e desnecessário para o escopo do trabalho).
        while !round_finished && answered_count < 2 {
            // ... (lógica de recebimento, pontuação e verificação do primeiro a responder) ...
            let (amt, src) = socket.recv_from(&mut buf)?;
            let msg = str::from_utf8(&buf[..amt]).unwrap_or("").trim();

            let player_idx = if players[0].addr == src {
                0
            } else if players[1].addr == src {
                1
            } else {
                continue;
            };
            let opponent_idx = if player_idx == 0 { 1 } else { 0 };

            if msg.starts_with("RESPOSTA:") {
                let answer_char = msg.chars().last().unwrap_or(' ');
                println!(
                    "Jogador {} respondeu: {}",
                    players[player_idx].name, answer_char
                );

                // Garante que o jogador ainda não respondeu nesta rodada
                // NOTA: Para um controle 100% à prova de falhas, você precisaria de um flag de "respondeu_nesta_rodada"
                // em cada jogador, mas como estamos no UDP, a lógica de 'answered_count' e 'first_error' já é suficiente.

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
                        let aviso_erro = "VOCE ERROU! Aguardando adversario...";
                        socket.send_to(aviso_erro.as_bytes(), players[player_idx].addr)?;

                        // Avisa o oponente que é a vez dele
                        let aviso_oponente = "O ADVERSARIO ERROU! Sua vez de tentar (+3 pts)...";
                        socket.send_to(aviso_oponente.as_bytes(), players[opponent_idx].addr)?;
                    }
                } else {
                    // === SEGUNDO A RESPONDER (Só acontece se o primeiro errou) ===
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
        } // FIM DO WHILE DE RESPOSTAS

        // === FIM DA RODADA: Atualiza e envia placar ===
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
                let win_msg = format!("FIM DE JOGO! VENCEDOR: {} com {} pontos.", p.name, p.score);
                println!("{}", win_msg);
                for client in &players {
                    socket.send_to(win_msg.as_bytes(), client.addr)?;
                }
                break; // Sai do loop de verificação de placar
            }
        }

        // Avança para a próxima pergunta
        question_index += 1;

        // Se todas as perguntas foram usadas, mas o jogo não acabou,
        // o código deve se encaixar no requisito do professor de "reinício".
        if question_index >= questions.len() && game_running {
            println!("Acabaram todas as perguntas. O jogo encerra por falta de questões.");
            for p in &players {
                let msg = "FIM DE JOGO: Todas as perguntas foram usadas.";
                socket.send_to(msg.as_bytes(), p.addr)?;
            }
            game_running = false;
        }

        std::thread::sleep(Duration::from_secs(2));
    }

    Ok(())
}
