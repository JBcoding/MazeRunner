"""
Ricochet Robots Solver in Python3.

Asger Juul Brunshøj
s103412@student.dtu.dk

TODO:
Might want to store move_history as a linked list instead of copies.
Creating a state takes too long if it turns out to have already been seen.
Find a way to cut down an entire tree if no children are making significant progress.
branch only one robot at a time.
"""
import collections
import heapq
import os
import sys


# DEBUG: Print additional info on the solution
DEBUG = True

# PREMATURE: Set to x to stop at the x'th seen state and output sequence of
# moves that look most promising at that time. Set to zero to turn off. DEBUG
# must be set to True.
PREMATURE = 0

# SUPPRESS: Suppress solution output.
SUPPRESS = False
SUPPRESS_ORG = True

# DIST_OUTPUT: This run is only intended to output the distance board.
DIST_OUTPUT = False

# PLOTTING: Use matplotlib to visualize solution paths realtime. DEBUG must be
# true
PLOTTING = True


if DEBUG:
    import time


_field_names = [
    'primary_robot_steps',  # (int) Number of steps taken by the primary robot
    'secondary_robot_steps',  # (int) Number of steps taken by secondary robots
    'sec_robots_in_use',  # (tuple) secondary robots involved
    'move_history',  # (tuple) Sequence of moves, '1U', that led to this state
    'robot_positions',  # (tuple) Where are the robots ((x0,y0), (x1,y1), ...)
    'last_move',  # (tuple) Last move taken (robot (int), direction (str))
    'visited_by_zero',  # (frozenset) squares visited by robot 0 {(row, col), }
]
StateTuple = collections.namedtuple('State', _field_names)


class AlreadySeenError(Exception):

    pass


class BadMoveError(Exception):

    pass


class Cell():
    """Holds up to four instance attributes: 'U', 'D', 'L' and 'R'.

    Attributes hold tuples (row, col) indicating an adjacent cell in a given
    direction.
    """

    def set(self, direction, adjacent_cell):
        if direction == 'U':
            self.U = adjacent_cell
        elif direction == 'D':
            self.D = adjacent_cell
        elif direction == 'L':
            self.L = adjacent_cell
        elif direction == 'R':
            self.R = adjacent_cell

    def get(self, direction):
        if direction == 'U':
            return self.U
        elif direction == 'D':
            return self.D
        elif direction == 'L':
            return self.L
        elif direction == 'R':
            return self.R


class State():
    """A state with properties such as the moves to get here, and some methods.

    The methods can yield child states in order to branch out the solution
    attempts.
    """

    def __init__(self, statetuple):
        self.statetuple = statetuple

    @staticmethod
    def from_parent(parent, robot, direction, steps, force=False):
        """Create a child state based on a parent state.

        Args:
            parent (State): Parent state.
            robot (int): The robot involved in the move.
            direction (str): 'U', 'D', 'L' or 'R'.
            steps (int): Number of steps involved in the move.
            force (bool): Do not check if already seen.

        Returns:
            State: Child state.

        Raises:
            AlreadySeenError: If cannot create state, because it has already
                been seen.
        """
        # New robot positions:
        pst = parent.statetuple
        prp = pst.robot_positions
        row, col = prp[robot]
        if direction == 'U':
            new_robot_pos = (row - steps, col)
        elif direction == 'D':
            new_robot_pos = (row + steps, col)
        elif direction == 'L':
            new_robot_pos = (row, col - steps)
        elif direction == 'R':
            new_robot_pos = (row, col + steps)
        rp = tuple(prp[:robot]) + (new_robot_pos, ) + tuple(prp[1 + robot:])

        # Check if already seen
        if not force and SeenStates.seen(rp):
            raise AlreadySeenError()

        if robot == 0:
            primary_robot_steps = pst.primary_robot_steps + 1
            secondary_robot_steps = pst.secondary_robot_steps
        else:
            primary_robot_steps = pst.primary_robot_steps
            secondary_robot_steps = pst.secondary_robot_steps + 1

        move = str(robot) + direction
        move_history = pst.move_history + (move, )

        pss = pst.sec_robots_in_use
        if robot == 0 or robot in pss:
            sec_robots_in_use = pss
        else:
            sec_robots_in_use = pss + (robot, )

        last_move = (robot, direction)

        visited_by_zero = pst.visited_by_zero.union((new_robot_pos, ))

        kwargs = {
            'primary_robot_steps': primary_robot_steps,
            'secondary_robot_steps': secondary_robot_steps,
            'sec_robots_in_use': sec_robots_in_use,
            'move_history': move_history,
            'robot_positions': rp,
            'last_move': last_move,
            'visited_by_zero': visited_by_zero,
        }
        statetuple = StateTuple(**kwargs)
        return State(statetuple)

    def yield_child_states(self, robot=None):
        """Generator for all possible descendant states in 1 move.

        Args:
            robot (int): Only branch on a specific robot.

        Yields:
            State: Child states for all possible directions.
        """
        if robot is None:
            # default
            robot_range = range(ROBOTS)
        else:
            robot_range = (robot, )
        for rob in robot_range:
            last_move_robot, last_move_direction = self.statetuple.last_move
            if last_move_robot != rob:
                directions = 'UDLR'
            elif last_move_direction in 'UD':
                directions = 'LR'
            else:
                directions = 'UD'
            for direction in directions:
                if self.statetuple.last_move != (rob, direction):
                    try:
                        child_state = create_child_state(self, rob, direction)
                    except BadMoveError:
                        pass
                    except AlreadySeenError:
                        pass
                    else:
                        yield child_state


class SeenStates():
    seen_states = set()

    @classmethod
    def seen(cls, robot_positions):
        """Add a state to the set of seen states.

        Args:
            robot_positions (tuple): Robot positions.

        Returns:
            boolean: True if the state has been seen previously, otherwise
                False.
        """
        rp = robot_positions

        prim = rp[0]

        # Make no distinction between secondary robots:
        sec = frozenset(rp[1:])

        # Some alternative approaches that were not as fast:
        # sec = tuple(sorted(rp[1:]))
        # sec = tuple(sorted(row * DIMENSION + col for row, col in rp[1:]))
        # sec = frozenset(complex(row, col) for row, col in rp[1:])
        # sec = 0
        # for row, col in rp[1:]:
        #     field = 2 ** (row * DIMENSION + col)
        #     sec = sec | field

        frozen_state = (prim, sec)
        if frozen_state in cls.seen_states:
            return True
        else:
            cls.seen_states.add(frozen_state)
            row, col = prim
            R0_DENSITY[row][col] += 1
            return False


def check_if_we_won(state):
    """Check if robot zero came to the goal."""
    return state.statetuple.robot_positions[0] == GOAL


def conclude(state):
    """Conclude, based on state.

    Purges the solution and prints solutions to stdout.
    """
    winning_move_hist = state.statetuple.move_history
    purged_move_hist = post_process(winning_move_hist)
    if not SUPPRESS:
        print_solution(purged_move_hist)
    if DEBUG:
        if not SUPPRESS and not SUPPRESS_ORG:
            print('== Original solution: ==')
            print_solution(winning_move_hist)
        print('    Original solution in', len(winning_move_hist), 'Moves.')
        print('    Purged solution in', len(purged_move_hist), 'Moves.')
        print('    Seen states:', len(SeenStates.seen_states))
        print('    Time:', time.time() - t0)
    sys.stdout.flush()
    os._exit(0)


def print_solution(move_history):
    """Print the move history."""
    for move in move_history:
        print(move)


def create_child_state(state, robot, direction, force=False):
    """I don't remember how this function works anymore.

    I hope you like spaghetti.

    Args:
        robot (int): Robot to move.
        direction (str): 'U', 'D', 'L' or 'R'.
        force (bool): Does not check for invalid/bad moves.

    Returns:
        State: A new instance of the State class.

    Raises:
        BadMoveError: Raised when a move results in a previously seen state, or
            it is not possible to move the robot in this direction because
            of a wall or another robot standing in the way.
        AlreadySeenError: state already seen.
    """
    # Current position of the robot
    robot_positions = state.statetuple.robot_positions
    rob_row, rob_col = robot_positions[robot]
    try:
        adj_row, adj_col = BOARD[rob_row][rob_col].get(direction)
    except AttributeError:
        if force:
            adj_row, adj_col = rob_row, rob_col
        else:
            raise BadMoveError()
    # Check if other robots are in the way
    if direction == 'U':
        steps = rob_row - adj_row
    elif direction == 'D':
        steps = adj_row - rob_row
    elif direction == 'L':
        steps = rob_col - adj_col
    elif direction == 'R':
        steps = adj_col - rob_col
    for sec_row, sec_col in robot_positions:
        if direction == 'U':
            cond = rob_col == sec_col and rob_row > sec_row >= adj_row
        elif direction == 'D':
            cond = rob_col == sec_col and rob_row < sec_row <= adj_row
        elif direction == 'L':
            cond = rob_row == sec_row and rob_col > sec_col >= adj_col
        elif direction == 'R':
            cond = rob_row == sec_row and rob_col < sec_col <= adj_col
        if cond:
            # The robot is in the way
            if direction == 'U':
                if sec_row == rob_row - 1 and not force:
                    raise BadMoveError()
                adj_row = sec_row + 1
                steps = rob_row - adj_row
            elif direction == 'D':
                if sec_row == rob_row + 1 and not force:
                    raise BadMoveError()
                adj_row = sec_row - 1
                steps = adj_row - rob_row
            elif direction == 'L':
                if sec_col == rob_col - 1 and not force:
                    raise BadMoveError()
                adj_col = sec_col + 1
                steps = rob_col - adj_col
            elif direction == 'R':
                if sec_col == rob_col + 1 and not force:
                    raise BadMoveError()
                adj_col = sec_col - 1
                steps = adj_col - rob_col

    child_state = State.from_parent(state, robot, direction, steps, force=force)
    return child_state


def post_process(move_history):
    """Remove unnecessary moves by secondary robots from the solution.

    mmh stands for modified_move_history.
    smh stands for shorter_move_history. It is the shortest found solution so
    # far
    """
    if DEBUG:
        print('post processing!')
        t0 = time.time()
    smh = move_history

    # Attempt to remove entire robots from the move_history:
    sec_robots_in_solution = {move[0] for move in smh}
    for robot in sec_robots_in_solution:
        mmh = [move for move in smh if move[0] != robot]
        if solution_is_valid(mmh):
            smh = mmh

    # Step through move_history backwards and attempt to remove unnecessary
    # moves.
    # Here robot zero is not involved.
    # The rationale here is that some robots might continue to move
    # around after they have been useful, and that this is faster than the
    # every move approach below. So we do this first to cut the move_history
    # as much as possible before trying every move.
    necessary_robots = [0]
    modification = True
    while modification:
        modification = False
        for i in range(len(smh) - 1, -1, -1):
            # loop through smh backwards
            robot = int(smh[i][0])
            if robot not in necessary_robots:
                # skip move i:
                mmh = smh[:i] + smh[1 + i:]
                modification = True
                if solution_is_valid(mmh):
                    smh = mmh
                    break
                else:
                    necessary_robots.append(robot)
                    break

    # Step through forwards and remove unnecessary moves.

    necessary_robots = [0]
    modification = True
    while modification:
        i = 0
        modification = False
        # loop through once:
        while i < len(smh):
            # skip move i:
            mmh = smh[:i] + smh[1 + i:]
            if solution_is_valid(mmh):
                modification = True
                smh = mmh
            else:
                i += 1

    # Step through solution and find out if any single move can be removed,
    # restarting from the front every time a move is removed. This is SLOW:
    modification = True
    while modification:
        modification = False
        for i, move in enumerate(smh):
            mmh = smh[:i] + smh[1 + i:]
            if solution_is_valid(mmh):
                modification = True
                smh = mmh
                break

    if DEBUG:
        print('Finished purging solution in', time.time() - t0, 'sec')

    return smh


def solution_is_valid(move_history):
    """Find out if a given history of moves produces a valid solution.

    Returns True/False
    """
    state = INIT_STATE
    for robot, direction in move_history:
        robot = int(robot)
        state = create_child_state(state, robot, direction, force=True)
    won = check_if_we_won(state)
    return won


def parse():
    """Parse the text representation of the board."""
    dim = int(sys.stdin.readline())
    robots = int(sys.stdin.readline())

    robot_pos = [None] * robots

    raw_board = []
    for row, line in enumerate(sys.stdin.readlines()):
        raw_line = [col if col in '# ' else ' ' for col in line.rstrip('\n')]
        raw_board.append(raw_line)
        for col, char in enumerate(line):
            if char in '0123456789':
                robot_pos[int(char)] = (row, col)
            if char == 'G':
                goal = (row, col)

    return {
        'dim': dim,
        'robots': robots,
        'raw_board': raw_board,
        'robot_pos': tuple(robot_pos),
        'goal': goal
    }


def distance_metric(state):
    """
    A ranking for a state to determine if it is worth pursuing further.

    Distance metric is computed from:
    * Distance to goal, weighted less for secondary robots.
    * Number of steps taken, weighted more for secondary robots.
    * Number of robots in use.
    * Number of recent moves that did not involve the primary robot.
    * Number of seen states that share the position of robot zero.
    """

    # ### PARAMETERS ###
    n_robots_weight = 1

    # bfs_layers = 0

    dist_weight = 2
    dist_sec_weight = 0.3

    steps_weight = 0.3
    steps_sec_weight = 1.5

    moves_since_robot_zero_weight = 0

    similar_states_weight = 0

    diversity_weight = 0.2

    # ### COMPUTATION ###
    s = state.statetuple
    dist = 0

    moves_since_primary = 0
    for robot, direction in reversed(s.move_history):
        if robot != 0:
            moves_since_primary += 1
    dist += moves_since_primary * moves_since_robot_zero_weight

    # Force BFS for the first couple of iterations:
    # if s.primary_robot_steps + s.secondary_robot_steps <= bfs_layers:
    #     return dist

    # Robot 0
    # dist += labyrinth_dist(*s.robot_positions[0]) * dist_weight
    r_row, r_col = s.robot_positions[0]
    dist += DIST[r_row][r_col] * dist_weight

    # Other robots
    for r_row, r_col in s.robot_positions[1:]:
        # dist += labyrinth_dist(r_row, r_col) * dist_sec_weight
        dist += DIST[r_row][r_col] * dist_sec_weight

    # Steps
    dist += s.primary_robot_steps * steps_weight
    dist += s.secondary_robot_steps * steps_sec_weight

    # Robots in use
    dist += len(s.sec_robots_in_use) * n_robots_weight

    # Other states sharing same position for robot zero
    row, col = state.statetuple.robot_positions[0]
    dist += R0_DENSITY[row][col] * similar_states_weight

    # Try to force robot zero to visit new squares
    # (Counted negatively)
    dist -= len(s.visited_by_zero) * diversity_weight

    return dist


def pre_process_distance(raw_board):
    """Pre-process the distance function."""
    dist = [[-1 for _ in range(DIMENSION)] for _ in range(DIMENSION)]
    visited = [[False for _ in range(DIMENSION)] for _ in range(DIMENSION)]

    # BFS search from the goal:
    q = collections.deque()
    q.append(GOAL)
    dist[GOAL[0]][GOAL[1]] = 0

    n = DIMENSION
    rb = raw_board
    while q:
        r, c = q.popleft()
        if 0 <= r + 1 < n and not visited[r + 1][c] and rb[r + 1][c] == ' ':
            # Down
            dist[r + 1][c] = dist[r][c] + 1
            q.append((r + 1, c))
            visited[r + 1][c] = True
        if 0 <= r - 1 < n and not visited[r - 1][c] and rb[r - 1][c] == ' ':
            # Up
            dist[r - 1][c] = dist[r][c] + 1
            q.append((r - 1, c))
            visited[r - 1][c] = True
        if 0 <= c + 1 < n and not visited[r][c + 1] and rb[r][c + 1] == ' ':
            # Right
            dist[r][c + 1] = dist[r][c] + 1
            q.append((r, c + 1))
            visited[r][c + 1] = True
        if 0 <= c - 1 < n and not visited[r][c - 1] and rb[r][c - 1] == ' ':
            # Left
            dist[r][c - 1] = dist[r][c] + 1
            q.append((r, c - 1))
            visited[r][c - 1] = True

    # if a robot is on a square of -1, then it is useless:
    for robot, pos in enumerate(INIT_ROBOT_POS):
        row, col = pos
        if dist[row][col] == -1:
            FROZEN_ROBOTS.append(robot)

    return dist


def pre_process_graph(raw_board):
    """Pre-process the raw string board into a graph-like representation.

    A cell points directly to other cells that are reachable, given that no
    robot stands in the way.
    """
    board = [[Cell() for _ in range(DIMENSION)] for _ in range(DIMENSION)]

    direction = 'U'
    for col in range(DIMENSION):
        streak = 0
        for row in range(DIMENSION):
            char = raw_board[row][col]
            if char == '#':
                streak = 0
                continue
            if streak > 0:
                adj = (row - streak, col)
                board[row][col].set(direction, adj)
            streak += 1

    direction = 'D'
    for col in range(DIMENSION):
        streak = 0
        for row in range(DIMENSION - 1, -1, -1):
            char = raw_board[row][col]
            if char == '#':
                streak = 0
                continue
            if streak > 0:
                adj = (row + streak, col)
                board[row][col].set(direction, adj)
            streak += 1

    direction = 'L'
    for row in range(DIMENSION):
        streak = 0
        for col in range(DIMENSION):
            char = raw_board[row][col]
            if char == '#':
                streak = 0
                continue
            if streak > 0:
                adj = (row, col - streak)
                board[row][col].set(direction, adj)
            streak += 1

    direction = 'R'
    for row in range(DIMENSION):
        streak = 0
        for col in range(DIMENSION - 1, -1, -1):
            char = raw_board[row][col]
            if char == '#':
                streak = 0
                continue
            if streak > 0:
                adj = (row, col + streak)
                board[row][col].set(direction, adj)
            streak += 1

    return board


def branch(state):
    """Generator that produces layers of child states. Basically like BFS.

    This allows the algo to perform some bad moves, in case the next move
    immediately after is a good one.

    The children in the first levels are being exhausted and do not need to go
    on the heap.
    """
    # levels = 1 would essentially disable the functionality in this function.

    # s = state.statetuple
    # state_moves = s.primary_robot_steps + s.secondary_robot_steps
    # levels = state_moves + 1

    levels = 5
    zero_levels = 2

    all_output_states = []
    states_for_one_robot = [[state] for _ in range(ROBOTS)]
    for level in range(levels + zero_levels):
        if level > levels:
            # switch to robot zero
            robot_range = (0, )
        else:
            robot_range = range(ROBOTS)
        for robot in robot_range:
            if robot in FROZEN_ROBOTS:
                continue
            child_states = []
            for state in states_for_one_robot[robot]:
                child_states += list(state.yield_child_states(robot=robot))
            for child in child_states:
                if check_if_we_won(child):
                    conclude(child)
                all_output_states.append(child)
            states_for_one_robot[robot] = child_states

    return all_output_states


if DEBUG:
    t0 = time.time()

# Get input
_p = parse()

# Frozen robots are robots that are not allowed to move.
FROZEN_ROBOTS = []

DIMENSION = _p['dim']
ROBOTS = _p['robots']
INIT_ROBOT_POS = _p['robot_pos']
# Position of the goal as a tuple, (row, col):
GOAL = _p['goal']

# in how many states have robot zero occupied some square
R0_DENSITY = [[0 for _ in range(DIMENSION)] for _ in range(DIMENSION)]

BOARD = pre_process_graph(_p['raw_board'])
DIST = pre_process_distance(_p['raw_board'])
if DIST_OUTPUT:
    print(DIST)
    sys.stdout.flush()
    os._exit(0)

kwargs = {
    'primary_robot_steps': 0,
    'secondary_robot_steps': 0,
    'sec_robots_in_use': (),
    'move_history': (),
    'robot_positions': INIT_ROBOT_POS,
    'last_move': (None, None),
    'visited_by_zero': frozenset((INIT_ROBOT_POS[0]), ),
}
_init_statetuple = StateTuple(**kwargs)
INIT_STATE = State(_init_statetuple)
SeenStates.seen(INIT_STATE.statetuple.robot_positions)
init_dist = distance_metric(INIT_STATE)


"""Go explore the board and look for a solution."""
states_counter = 0  # heapq is not stable without this.
first_item = (init_dist, states_counter, INIT_STATE)
heap = []
heapq.heappush(heap, first_item)

# Forget old states to free up RAM:
target_heap_size = 2 ** 14


if DEBUG and PLOTTING:
    import matplotlib.pyplot as plt
    import copy
    plot_handle = plt.plot([], [], 'k')
    static = [[0 for _ in range(DIMENSION)] for _ in range(DIMENSION)]

    # walls:
    for row_no, row in enumerate(_p['raw_board']):
        for col_no, char in enumerate(row):
            if char == '#':
                static[row_no][col_no] = 1

    # goal:
    static[GOAL[0]][GOAL[1]] = 2

    ax = plt.gca()
    ax.invert_yaxis()
    plt.show(block=False)
    # plt.ion()

it = 0
while True:
    it += 1
    try:
        lowest_dist, _, priority_state = heapq.heappop(heap)
    except IndexError:
        print('I give up!')
        sys.stdout.flush()
        os._exit(0)

    if DEBUG and it % 1000 == 0:
        seen_states = len(SeenStates.seen_states)
        print('Seen states:', seen_states)
        if 0 < PREMATURE < seen_states:
            best_bet = priority_state.statetuple.move_history
            print('==PREMATURE OUTPUT==')
            if not SUPPRESS:
                print_solution(best_bet)
            sys.stdout.flush()
            os._exit(0)

    for child_state in branch(priority_state):
        dist = distance_metric(child_state)
        states_counter += 1
        heapq.heappush(heap, (dist, states_counter, child_state))

    if len(heap) > 2 * target_heap_size:
        if DEBUG:
            print('Cutting heap in half! Size of heap was:', len(heap))
        heap = heap[:target_heap_size]

    if DEBUG and PLOTTING:
        static_mod = copy.deepcopy(static)
        pss = priority_state.statetuple

        # robot zero
        row, col = pss.robot_positions[0]
        static_mod[row][col] = 3

        # other robots
        for row, col in pss.robot_positions[1:]:
            static_mod[row][col] = 4

        ax.clear()
        ax.imshow(static_mod, interpolation='none', aspect=1)
        plt.draw()
        plt.pause(0.001)
