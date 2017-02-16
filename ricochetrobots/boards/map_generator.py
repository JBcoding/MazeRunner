import os
import copy
import random


def main():
    # very_easy_boards()
    #random_boards()
    large_random_boards()

def very_easy_boards():
    density = 0.10
    for n in [5, 7, 10]:
        for robots in [1, 2, 3, 4]:
            for difficulty_target in [1, 2, 3]:
                generate(n, robots, difficulty_target, density)

def random_boards():
    density = 0.10
    for n in [5, 6, 7, 8, 9, 10, 12, 15, 20, 25, 30, 45, 60, 80, 100]:
        for robots in [1, 2, 4, 8]:
            for difficulty_target in set((5, min(n, 20))):
                generate(n, robots, difficulty_target, density)
				
def large_random_boards():
    density = 0.30
    for n in [150, 250]:
        for robots in [4]:
            for difficulty_target in [min(n, 50)]:
                generate(n, robots, difficulty_target, density)

def large_boards():
    density = 0.10
    for n in [5]:
        for robots in [1, 2, 4, 8]:
            for difficulty_target in set((5, min(n, 20))):
                generate(n, robots, difficulty_target, density)

def generate(n, robots, difficulty_target, density):
    foldername = 'newly_generated'
    if robots > 1 and difficulty_target == 1:
        # Not possible to create a board with a one move solution using more than one robot
        return
    board, difficulty = generate_board(n=n, density=density, robots=robots, func=random_board, difficulty_target=difficulty_target)
    board_name = 'n=' + str(n) + '___robots=' + str(robots) + '___possible_in_' + str(difficulty) + '_moves'
    export_board(board=board, folder=foldername, filename=board_name, n=n, robots=robots)

def export_board(board, folder, filename, n, robots):
    if not os.path.exists(folder):
        os.makedirs(folder)
    path = os.path.join(folder, filename + '.txt')
    print('Exporting', path)
    with open(path, 'w') as f:
        f.write(str(n) + '\n')
        f.write(str(robots) + '\n')
        for row in board:
            for col in row:
                f.write(col)
            f.write('\n')

def generate_board(n, density, robots, func, difficulty_target):
    c = 1
    attempts_per_difficulty = 100
    while True:
        print('\rGenerating board n={} robots={} difficulty_target={} attempt={}/{}'.format(n, robots, difficulty_target, c, attempts_per_difficulty), end='', flush=True)
        try:
            board = func(n=n, density=density)
            board = place_robots_and_goal(board, robots, n, difficulty_target)
            print()
            return board, difficulty_target
        except AssertionError:
            c += 1
            if c > attempts_per_difficulty:
                c = 0
                difficulty_target -= 1

def random_board(n, density):
    """
    type must be one of 'single' or 'multi'
    'single' : This board can be solved by only moving robot 0
    'multi' : This board MAY only be solvable by using multiple robots
    """
    board = [[' ']*n for _ in range(n)]

    ### Place blocks
    for row in range(n):
        for col in range(n):
            if random.random() <= density:
                board[row][col] = '#'
    return board

def place_robots_and_goal(board, robots, n, difficulty_target):
    instance = copy.deepcopy(board)

    ### Place robots
    robot_placements = [None] * robots
    for robot in range(robots):
        c = 0
        while True:
            c += 1
            row = random.randint(0, n-1)
            col = random.randint(0, n-1)
            if instance[row][col] == ' ':
                instance[row][col] = str(robot)
                robot_placements[robot] = (row, col)
                break
            assert c < 100 # avoid infinity loop if we are really unlucky or density is set way too high

    ### Place goal
    # Attempt to pick an interesting goal position by simulating robots moving about randomly.
    # Simulate a number of times. Keep track of where robot zero has been. At the end, pick
    # a square where robot 0 has not been often.
    if n <= 100:
        runs = min(n*1000, 10000)
    else:
        # just takes too long otherwise...
        runs = 1000
    robot_zero_sim_endpos = {}
    trivial = set() # set of goal positions considered too easy
    for _ in range(runs):
        instance_temp = copy.deepcopy(instance)
        robot_placements_temp = copy.deepcopy(robot_placements)
        for move in range(difficulty_target):
            if robots == 1:
                robot = 0
            else:
                robot = random.randint(0, robots - 1)
            direction = random.sample({'up', 'down', 'left', 'right'}, 1)[0]
            old_row, old_col = robot_placements_temp[robot]
            new_row, new_col = robot_placements_temp[robot]
            while 0 <= new_row < n and 0 <= new_col < n and instance_temp[new_row][new_col] in {' ', str(robot)}:
                robot_placements_temp[robot] = new_row, new_col
                if direction == 'up':
                    new_col -= 1
                elif direction == 'down':
                    new_col += 1
                elif direction == 'left':
                    new_row -= 1
                elif direction == 'right':
                    new_row += 1
            instance_temp[old_row][old_col] = ' '
            new_row, new_col = robot_placements_temp[robot]
            instance_temp[new_row][new_col] = str(robot)

            #  Mark as valid possible goal position
            if robot == 0:
                if instance[new_row][new_col] == ' ':
                    if (new_row, new_col) in robot_zero_sim_endpos:
                        robot_zero_sim_endpos[(new_row, new_col)] += 1
                    else:
                        robot_zero_sim_endpos[(new_row, new_col)] = 1
                    if move + 1 < difficulty_target:
                        # All except the last move
                        trivial.add((new_row, new_col))

    assert len(robot_zero_sim_endpos) > 0 # might fail if robot zero is completely locked in by an unlucky spawn fx.
    assert set(robot_zero_sim_endpos) - trivial != set() # avoid boring boards

    robot_zero_sim_endpos = {k: v for k, v in robot_zero_sim_endpos.items() if k not in trivial}
    # while min(robot_zero_sim_endpos, key=robot_zero_sim_endpos.get) in trivial:
        # disregard goal position if it is in the set trivial.
        # del robot_zero_sim_endpos[min(robot_zero_sim_endpos, key=robot_zero_sim_endpos.get)]

    # find the goal
    goal_row, goal_col = min(robot_zero_sim_endpos, key=robot_zero_sim_endpos.get)
    instance[goal_row][goal_col] = 'G'

    # Now try to only move robot zero, and make sure that at least there is not a trivial solution that does not utilize the other robots..
    if robots > 1:
        for run in range(runs):
            instance_temp = copy.deepcopy(instance)
            robot_placements_temp = copy.deepcopy(robot_placements)
            for move in range(difficulty_target):
                robot = 0
                direction = random.sample({'up', 'down', 'left', 'right'}, 1)[0]
                old_row, old_col = robot_placements_temp[robot]
                new_row, new_col = robot_placements_temp[robot]
                while 0 <= new_row < n and 0 <= new_col < n and instance_temp[new_row][new_col] in {' ', 'G', str(robot)}:
                    robot_placements_temp[robot] = new_row, new_col
                    if direction == 'up':
                        new_col -= 1
                    elif direction == 'down':
                        new_col += 1
                    elif direction == 'left':
                        new_row -= 1
                    elif direction == 'right':
                        new_row += 1
                instance_temp[old_row][old_col] = ' '
                new_row, new_col = robot_placements_temp[robot]
                instance_temp[new_row][new_col] = str(robot)

                # make sure that robot zero has not reached the goal
                assert (new_row, new_col) != (goal_row, goal_col)

    return instance

def print_board(board):
    for row in board:
        print(*row, sep='')

if __name__ == '__main__':
    main()
