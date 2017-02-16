#include <iostream>
#include <vector>
#include <sstream>

using namespace std;

using Point = pair<int, int>;

string show_point(Point p);

enum class Direction: char {
    Up = 'U',
    Down = 'D',
    Left = 'L',
    Right = 'R'
};

struct Endpoints {
public:
    const Point up, down, left, right;

    string to_string();
};

class Board {
private:
    vector<vector<char>> board;
    size_t size;
    vector<Point> robots;
    Point goal;

    bool within_board(int x, int y) {
        return x >= 0 && x < this->size && y >= 0 && y < this->size;
    }

public:
    static const char GOAL_CHAR = 'G';
    static const char WALL_CHAR = '#';
    static const char EMPTY_CHAR = ' ';

    Board(vector<vector<char>> b, vector<Point> r, Point g);

    Endpoints possible_endpoints_for_robot(size_t robot);

    Point point_after_moving_robot(size_t robot, Direction m);

    void move_robot(size_t robot, Direction d);

    string to_string();

};

Board loadBoard();

int main() {
    Board board = loadBoard();

    cerr << board.to_string() << endl;

    cerr << "Possible endpoints for robot 0: " << board.possible_endpoints_for_robot(0).to_string() << endl;

    board.move_robot(0, Direction::Left);
	cout << 0 << (char)Direction::Left << endl;
	board.move_robot(0, Direction::Down);
	cout << 0 << (char)Direction::Down << endl;
	
    cerr << board.to_string() << endl;

    return 0;
}

Board::Board(vector<vector<char>> b, vector<Point> r, Point g) {
    this->board = b;
    this->size = b.size();
    this->robots = r;
    this->goal = g;
}

Endpoints Board::possible_endpoints_for_robot(size_t robot) {
    Point up = point_after_moving_robot(robot, Direction::Up);
    Point down = point_after_moving_robot(robot, Direction::Down);
    Point left = point_after_moving_robot(robot, Direction::Left);
    Point right = point_after_moving_robot(robot, Direction::Right);

    return Endpoints {up, down, left, right};
}

Point Board::point_after_moving_robot(size_t robot, Direction m) {
    Point pos = Point(robots[robot]);
    int drow = 0, dcol = 0;

    if (m == Direction::Up) {
        drow = -1;
    } else if (m == Direction::Down) {
        drow = 1;
    }

    if (m == Direction::Left) {
        dcol = -1;
    } else if (m == Direction::Right) {
        dcol = 1;
    }

    while (within_board(pos.first + drow, pos.second + dcol) &&
           (board[pos.first+drow][pos.second+dcol] == EMPTY_CHAR
            || board[pos.first+drow][pos.second+dcol] == GOAL_CHAR)) {
        pos.first += drow;
        pos.second += dcol;
    }

    return pos;
}

void Board::move_robot(size_t robot, Direction d) {
    Point from = robots[robot];
    Point to = point_after_moving_robot(robot, d);
    this->board[from.first][from.second] = EMPTY_CHAR;
    robots[robot] = to;
    board[to.first][to.second] = static_cast<char>('0' + robot);
}

string Board::to_string() {
    stringstream out;

    for (auto row : this->board) {
        string str(row.begin(), row.end());
        out << str << endl;
    }

    return out.str();
}

string Endpoints::to_string() {
    stringstream out;

    out << "Endpoints (Up: " << show_point(this->up) << ", Down: " << show_point(this->down)
    << ", Left: " << show_point(this->left) << ", Right: " << show_point(this->right) << ")";

    return out.str();
}

string show_point(Point p) {
    stringstream out;
    out << "(" << p.first << ", " << p.second << ")";
    return out.str();
}

Board loadBoard() {
    string line;

    getline(cin, line);

    size_t size = stoul(line);

    getline(cin, line);

    size_t number_of_robots = stoul(line);

    vector<vector<char>> board(size, vector<char>(size));
    vector<Point> robots(number_of_robots);

    Point goal;
    bool found_goal = false;

    for (size_t i = 0; i < size; i++) {
        getline(cin, line);

        for (size_t j = 0; j < size; j++) {
            char c = line[j];

            if (c == Board::GOAL_CHAR) {
                goal = Point(i, j);
                found_goal = true;
            } else if (isdigit(c)) {
                robots[c - '0'] = Point(i, j);
            } else if (c != Board::EMPTY_CHAR && c != Board::WALL_CHAR) {
                cerr << "Error: Unknown character '" << c
                << "' on line " << (i+1) << ", column " << (j+1) << endl;
            }

            board[i][j] = c;
        }
    }

    if (!found_goal) {
        cerr << "Error: No goal found on board" << endl;
    }

    return Board(board, robots, goal);
}
