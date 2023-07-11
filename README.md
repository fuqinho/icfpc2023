# icfpc2023

Team Spica for ICFP Programming Contest 2023

# Solver

We tried to find the optimal solution by combining several solvers.

## Simulated annealing solver

The Simulated Annealing solver uses Simulated annealing (SA) to search for a solution.
This problem has no order in the arrangement of the musicians and many parts of the score function are smoothed with respect to the arrangement of the musicians,
SA is likely to find good solutions to this problem.

Neighborhood selection is important in SA. Our solver employed the following neighborhoods.

* One musician is moved. During the move, we discretized the coordinates and expected fast convergence by narrowing the search. As the search progressed, we decreased the distance traveled and the degree of discretization. We expected this to result in a rough search in the early stages and detailed optimization at the end of the search. If a musician conflicts with another musician when moving, an attempt is made to move to the very edge of contact. This is based on the assumption that it would be closer to optimal if the musicians were in close proximity to each other, and in the hope that the movement will not be biased in a particular direction.
* Swapping the positions of two musicians, since the probability of such an operation occurring in a move of one musician to a location where another musician is present is very low, so we have added such an operation.
* Change the volume of the musicians. Ultimately, it seems best to fix it at 0.0 or 1.0 depending on the positive or negative impact on the audience, but we thought it would contribute to smoother transitions in the search to have it automatically converge there as the SA progresses.
* Combine several operations. Since the recalculation of scores with updates was considered to be a relatively heavy process, we also considered the speed of convergence of the solution to be important. Specifically, we changed the position of two people at the same time, or exchanged two or more positions at the same time.

Since the computation of scores is very computationally intensive, we performed a differential update according to the changes. This resulted in an average of about `O(M+AlogA)` for the calculation of the score for a single person's position change, which is about several thousand calculations per second.

## ...

## ...

## Combining solvers

It is possible to improve the solution by performing SA multiple times, but eventually it will stalemate and stop improving. By inserting another type of solver, the stalemate may be slightly improved and further improvement may be expected. For problems with relatively high-impact improvement potential, we did this many times in an effort to increase the score as much as possible within a limited time frame.
