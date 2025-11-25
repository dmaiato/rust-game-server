# 🎮 Battle Quiz - Rust Game Server

A simple, zero-dependency, UDP-based 2-player quiz game server written in Rust.

[![Language](https://img.shields.io/badge/language-Rust-rust.svg?style=flat-square)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%20(TBD)-blue.svg?style=flat-square)](./LICENSE)

---

## 🚀 How it Works

The server manages a real-time quiz game for two players over UDP. The game flow is as follows:

1.  **Lobby:** The server starts and waits for two players to join on port `10000`.
2.  **Game Start:** Once two players have connected, the game begins, and questions are sent to both players simultaneously.
3.  **Answering:** Players submit their answers.
    *   The first player to answer correctly scores **+5 points**, and the round ends.
    *   If the first player answers incorrectly, the second player gets a chance to answer for **+3 points**.
    *   If both players answer incorrectly, no points are awarded.
4.  **Game End:** The game concludes when one of the players reaches a score of 30.

## 📡 Network Protocol

Communication is done via simple, plain-text UDP messages.

-   **Client to Server:**
    -   `LOGIN:<PlayerName>`: To join the lobby.
    -   `RESPOSTA:<A|B|C>`: To answer the current question.

-   **Server to Client:**
    -   `BEM-VINDO! ...`: Welcome message upon joining.
    -   `JOGO_INICIADO`: Sent to both players when the game starts.
    -   `PERGUNTA:...
A) ... | B) ...`: Sends the question and options.
    -   `PLACAR: ...`: Announces the end of the round and the current scores.
    -   `FIM DE JOGO! VENCEDOR: ...`: Announces the winner.

## 🛠️ Getting Started

### Prerequisites

-   [Rust Toolchain](https://rustup.rs/)

### Running the Server

1.  **Clone the repository:**
    ```sh
    git clone <your-repository-url>
    cd game-server
    ```

2.  **Run the server:**
    ```sh
    cargo run --release
    ```

The server will start and listen on `0.0.0.0:10000`.

### Playing the Game (using `ncat`)

Since there is no dedicated client, you can test the server using `ncat`, a powerful networking utility that is part of the [Nmap suite](https://nmap.org/ncat/).

1.  **Player 1 Terminal:**
    ```sh
    ncat --udp 127.0.0.1 10000
    LOGIN:PlayerOne
    ```

2.  **Player 2 Terminal:**
    ```sh
    ncat --udp 127.0.0.1 10000
    LOGIN:PlayerTwo
    ```

Once both players are in, the game will start. Type your answers (`RESPOSTA:A`, `RESPOSTA:B`, etc.) in the respective terminals.

## 🐳 Docker Support

The project includes files for containerization.

-   **Build the Docker image:**
    ```sh
    docker build -t battle-quiz-server .
    ```

-   **Run using Docker Compose:**
    ```sh
    docker-compose up
    ```

---

*This README was generated based on the project structure and source code.*
