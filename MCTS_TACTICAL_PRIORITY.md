# Design: MCTS Selection Formula Modification for Tactical Priority

## 1. Goal

To enhance the tactical awareness of the MCTS search within the humanlike engine by giving explicit priority to exploring checks and captures during the tree traversal (selection phase), while still leveraging the guidance from the Policy Network (P) and accumulated simulation results (Q).

## 2. Mechanism: Modified PUCT Selection Formula

The standard PUCT formula used for selecting the child node `a` to explore from parent node `s` will be modified to include an additive "Urgency Bonus" based on the move type.

## 3. Formula

**Standard PUCT:**

Select action `a` = argmax_a [ `Q(s,a)` + `C_puct` * `P(s,a)` * sqrt(`N(s)`) / (1 + `N(s,a)`) ]

Where:
*   `Q(s,a)` is the average action value (exploitation term).
*   `P(s,a)` is the prior probability from the Policy Network.
*   `N(s)` is the visit count of the parent node.
*   `N(s,a)` is the visit count of the child node.
*   `C_puct` is the exploration constant.

**Modified PUCT with Tactical Bonus:**

Select action `a` = argmax_a [ `Q(s,a)` + `C_puct` * `P(s,a)` * sqrt(`N(s)`) / (1 + `N(s,a)`) + `Bonus(a)` ]

## 4. Bonus Term Definition

The `Bonus(a)` term is defined as follows:

*   **If move `a` is a Check:**
    *   `Bonus(a)` = `check_urgency_bonus`
*   **If move `a` is a Capture:**
    *   `Bonus(a)` = `capture_urgency_bonus` * `scale(MVV_LVA(a))`
    *   `MVV_LVA(a)`: Score based on Most Valuable Victim - Least Valuable Aggressor heuristic.
    *   `scale()`: A function to normalize the `MVV_LVA(a)` score (e.g., to the range [0, 1] or another suitable range relative to Q-values and the exploration term).
*   **If move `a` is a Quiet move:**
    *   `Bonus(a)` = 0

## 5. Tunable Parameters

*   `check_urgency_bonus`: A constant value determining the priority boost for checks. Needs tuning.
*   `capture_urgency_bonus`: A constant value scaling the priority boost for captures based on their `MVV_LVA(a)` score. Needs tuning.
*   The `scale()` function for `MVV_LVA(a)` might also require adjustment.

Initial values should be small (e.g., 0.1 - 0.3) and adjusted based on testing.

## 6. Rationale

This approach cleanly integrates tactical priority into the existing MCTS selection mechanism. It allows the search to dynamically balance:
*   Exploiting known good moves (Q).
*   Exploring promising moves suggested by the policy network (P).
*   Investigating urgent tactical possibilities (Bonus).

It avoids the complexity of hybrid search algorithms or distorting the stored policy priors.

## 7. Integration Point

This modification applies specifically during the **Selection** phase of the MCTS algorithm, when traversing down the existing tree to find a leaf node to expand. The stored `P(s,a)` values remain the original priors from the Policy Network.