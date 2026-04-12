# Kudchuet

Kudchuet is a Rust framework for building board games, AI engines, and graphical game UIs using `egui`.

It provides:
- a generic board game abstraction
- pluggable AI engines (minimax, internal, external UCI engines)
- a modular UI system for rendering boards and pieces
- async AI move computation support
- evaluation and search utilities for game AI

---
## Live demos

- Chess: https://ultrasuperpingu.github.io/kudchuet/chess
- Chinese Checkers: https://ultrasuperpingu.github.io/kudchuet/chinese_checkers
- Three Musketeers: https://ultrasuperpingu.github.io/kudchuet/three_musketeers

## Features

### Board game framework
Define any turn-based board game by implementing:

- `BoardGame`
- `BoardMove`
- optional rendering traits (`BoardDrawer`, `SquareDrawer`)

Supports:
- Gomoku
- Chess-like engines (planned / extensible)
- custom abstract board games

---

### AI system

Kudchuet supports multiple AI backends:

#### Internal engines
- Minimax-based search (using the minimax crate with Iterative deepening and transposition table support)
- Expectiminimax for stractegic games with random parts.

#### External engines (UCI)
- If you provide a move and position serialization/desrialization, you get a simple uci server and gui implementation

### Custom rendering
You can override board rendering using ```BoardDrawer```, ```PieceDrawer``` and ```SquareDrawer```. Default implmentations of those are already really expressive but you can reimplement those and implement really specifc features.

## Example games

Kudchuet includes multiple fully playable example implementations:

### Regular 2 Players Grid Abstract strategy games
- Chess
- Connect Four
- Reversi (Othello)
- Checkers
- Abalone
- Gomoku
- Diaballik
- Yote

### Non Grid Board games
- Awale

### Asymmetric games
- Bagh-Chal
- Hare and Hounds
- Three Musketeers
- Neutron

### Dice / probabilistic games
- Backgammon

### Multiplayer games
- Chinese Checkers

Each game is implemented using the same `BoardGame` abstraction, and can be used with:
- internal AI engines (minimax / expectiminimax)
- external UCI engines
- the generic `egui` UI system

This makes Kudchuet a unified testbed for board game AI research and experimentation.
