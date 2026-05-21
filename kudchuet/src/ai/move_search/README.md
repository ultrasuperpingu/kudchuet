This part of the crate is a fork of the minimax crate:
https://github.com/edre/minimax-rs
It is published under the MIT license (see the LICENSE file), except the game_tree.rs and mcts.rs which are LGPL.

# Why a fork?

## Negamax vs Minimax

The original minimax crate uses the elegant Negamax formulation.
Negamax is compact and works beautifully for strictly two‑player, alternating, zero‑sum games such as chess, checkers, or Go.

However, kudchuet aims to provide a framework that supports as many strategic games as possible, including:
 * games with more than two players (e.g., Chinese Checkers)
 * games where a player may take multiple consecutive moves (e.g., Diabalik, Neutron)
 * games with stochastic transitions (dice, randomness)

In these settings, the Negamax assumption:
```
next_player = current_player.opponent
value(state, player) = -value(state, opponent)
```
no longer hold, because the “opponent” may not be unique, and the next player is not always known before the current move is completed.

For this reason, kudchuet uses a more explicit Minimax formulation, where evaluations are expressed directly from the perspective of the player whose move is being selected. This approach is more flexible and allows the framework to support a wider variety of games.
