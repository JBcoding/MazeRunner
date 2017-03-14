import java.awt.Point;
import java.io.BufferedReader;
import java.io.File;
import java.io.FileReader;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.StringReader;
import java.io.BufferedInputStream;
import java.io.FileInputStream;
import java.util.ArrayDeque;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.Map;
import java.util.Queue;
import java.util.Random;
import java.util.Set;
import java.util.StringTokenizer;

public class MazeRunner {

	final static char EMPTY_CHAR = ' ';
	final static char WALL_CHAR = '#';
	final static char GOAL_CHAR = '!';

	private Point goal;
	// Switches and the walls, which they affect
	private Map<String, Point> toggleSwitches = new HashMap<String, Point>();
	private Map<String, Point> holdSwitches = new HashMap<String, Point>();
	
	private Board board;
	
	public static void main(String[] args) {
		MazeRunner mr = new MazeRunner();
		
//		mr.parse(generateBoard(10, 10, 3, 2, 2, 8));
		mr.parse();
//		mr.computeSolution()
		
		String fileName = "output";
		int length = 0;
		try {
			length = countLines(fileName);
		} catch (IOException e1) {
			fail("Could not read solution.");
		}
		
		File file = new File(fileName);
		try {
			int result = mr.check(new BufferedReader(new FileReader(file)), length);
			if (result > 0) {
				System.out.println("RESULT CORRECT");
				System.out.println("SCORE " + 1000 / result);
			} else {
				fail("The solution did not reach the goal.");
			}
		} catch (IOException e) {
			fail("Could not read solution.");
		}
		
	}
	
	public static void fail(String reason) {
		System.out.println("RESULT WRONG");
		System.out.println("TEXT " + reason);
	}
	
	public static int countLines(String filename) throws IOException {
	    InputStream is = new BufferedInputStream(new FileInputStream(filename));
	    try {
	        byte[] c = new byte[1024];
	        int count = 0;
	        int readChars = 0;
	        boolean empty = true;
	        while ((readChars = is.read(c)) != -1) {
	            empty = false;
	            for (int i = 0; i < readChars; ++i) {
	                if (c[i] == '\n') {
	                    ++count;
	                }
	            }
	        }
	        return (count == 0 && !empty) ? 1 : count;
	    } finally {
	        is.close();
	    }
	}
	
	public String computeSolution() {
		String s = stringifyMoves(BFS());
		return s;
	}
	
	public ArrayList<Pair<Integer, Direction>> BFS() {		
		Set<Board> boards = new HashSet<Board>();
		Queue<Pair<Board, ArrayList<Pair<Integer, Direction>>>> q = 
				new ArrayDeque<Pair<Board, ArrayList<Pair<Integer, Direction>>>>();

		q.add(new Pair<Board, ArrayList<Pair<Integer, Direction>>>(this.board, new ArrayList<Pair<Integer, Direction>>()));
		boards.add(this.board);
		Board clearBoard = new Board(this.board.robots, copyMaze(this.board.maze));
		for (Point r : clearBoard.robots) {
			clearBoard.maze[r.x][r.y] = EMPTY_CHAR; // No need for robots on the reference course
		}
		
		while (!q.isEmpty()) {			
			Pair<Board, ArrayList<Pair<Integer, Direction>>> pair = q.poll();
			Board board = pair.fst;
			char[][] maze = board.maze;
			Point[] robots = board.robots;
			ArrayList<Pair<Integer, Direction>> moves = pair.snd;
						
			for (int r = 0; r < robots.length; r++) {
				
				Point robot = robots[r];
				List<Direction> neighbours = Arrays.asList(Direction.Up, Direction.Down, Direction.Left, Direction.Right);
				Collections.shuffle(neighbours);
				
				for (Direction d : neighbours) {
					
					Point newRobot = new Point(robot);
					switch (d) {
					case Up:	newRobot.translate(0, -1); break;
					case Down:	newRobot.translate(0, 1); break;
					case Left:	newRobot.translate(-1, 0); break;
					case Right:	newRobot.translate(1, 0); break;
					}
	
					if (!inBounds(newRobot)) continue;
					
					char c = board.maze[newRobot.x][newRobot.y];
					@SuppressWarnings("unchecked")
					ArrayList<Pair<Integer, Direction>> newMoves = (ArrayList<Pair<Integer, Direction>>)moves.clone();
					newMoves.add(new Pair<Integer, Direction>(r, d));				

					char[][] newMaze = copyMaze(maze);
					if (Character.isLetter(clearBoard.maze[robot.x][robot.y]) &&
							Character.isLowerCase(clearBoard.maze[robot.x][robot.y])) {
						Point hold = holdSwitches.get(String.valueOf(clearBoard.maze[robot.x][robot.y]));
						newMaze[hold.x][hold.y] = WALL_CHAR;
					}
					
					if (c == GOAL_CHAR) {
						return newMoves;
					} else if (c == EMPTY_CHAR || Character.isLetter(c)) {
						Point[] newRobots = Arrays.copyOf(robots, robots.length);
						newRobots[r] = newRobot;
						
						if (Character.isLetter(newMaze[newRobot.x][newRobot.y])) {
							if (Character.isUpperCase(clearBoard.maze[newRobot.x][newRobot.y])) {
								Point toggle = toggleSwitches.get(String.valueOf(clearBoard.maze[newRobot.x][newRobot.y]));
								newMaze[toggle.x][toggle.y] = newMaze[toggle.x][toggle.y] == WALL_CHAR ? EMPTY_CHAR : WALL_CHAR;
								if (newMaze[toggle.x][toggle.y] == EMPTY_CHAR) {
									for (int i = 0; i < robots.length; i++) {
										Point rob = robots[i];
										if (rob.equals(toggle)) {
											newMaze[toggle.x][toggle.y] = (char)((int)'0' + i);
											break;
										}
									}
								}
							} else if (Character.isLowerCase(clearBoard.maze[newRobot.x][newRobot.y])) {
								Point hold = holdSwitches.get(String.valueOf(clearBoard.maze[newRobot.x][newRobot.y]));
								newMaze[hold.x][hold.y] = EMPTY_CHAR;
								for (int i = 0; i < robots.length; i++) {
									Point rob = robots[i];
									if (rob.equals(hold)) {
										newMaze[hold.x][hold.y] = (char)((int)'0' + i);
										break;
									}
								}
							}
						}
						
						newMaze[newRobot.x][newRobot.y] = (char)((int)'0' + r);
						newMaze[robot.x][robot.y] = maze[robot.x][robot.y] == WALL_CHAR ? 
								WALL_CHAR : clearBoard.maze[robot.x][robot.y] == WALL_CHAR ? EMPTY_CHAR : clearBoard.maze[robot.x][robot.y];
						
						Board newBoard = new Board(newRobots, newMaze);
						
						if (boards.contains(newBoard)) continue;
						boards.add(newBoard);
						
						q.add(new Pair<Board, ArrayList<Pair<Integer, Direction>>>(newBoard, newMoves));
					}
				}
			}
		}
		
		return null;
	}
	
	private String stringifyMoves(ArrayList<Pair<Integer, Direction>> moves) {
		StringBuilder sb = new StringBuilder();
		for (Pair<Integer, Direction> pair : moves) {
			sb.append(pair.fst);
			sb.append(pair.snd);
			sb.append('\n');
		}
		return sb.toString();
	}

	private static char[][] copyMaze(char[][] maze) {
		char[][] newMaze = new char[maze.length][maze[0].length];
		for (int i = 0; i < maze.length; i++) {
			newMaze[i] = Arrays.copyOf(maze[i], maze[i].length);
		}
		return newMaze;
	}
	
	private boolean inBounds(Point p) {
		return p.x >= 0 && p.y >= 0 && p.x < board.maze.length && p.y < board.maze[0].length;
	}
	
	public int check(String s) {
		try {
			return check(new BufferedReader(new StringReader(s)));
		} catch (IOException e) {
			e.printStackTrace();
			return -1;
		}
	}
	
	public int check() {
		try {
			return check(new BufferedReader(new InputStreamReader(System.in)));
		} catch (IOException e) {
			e.printStackTrace();
			return -1;
		}
	}
	
	public int check(BufferedReader br) throws IOException {
		final int limit = 1000;
		return check(br, limit);
	}
	
	public int check(BufferedReader br, int moves) throws IOException {
		Point[] robots = board.robots;
		char[][] maze = copyMaze(board.maze);
		for (Point r : robots) {
			board.maze[r.x][r.y] = EMPTY_CHAR;
		}
		
		int steps = 0;
		for (int m = 0; m <= moves; m++) {
			char[] input = br.readLine().toCharArray();
			if (input.length < 2) continue;
			steps++;
			int r = (int)(input[0] - '0');
			Point robot = robots[r];
			char oldTile = board.maze[robot.x][robot.y];
			maze[robot.x][robot.y] = oldTile;
			
			switch (input[1]) {
			case 'U': robots[r].translate(0, -1); break;
			case 'D': robots[r].translate(0, 1); break;
			case 'L': robots[r].translate(-1, 0); break;
			case 'R': robots[r].translate(1, 0); break;
			}

			if (!inBounds(robots[r]) || maze[robot.x][robot.y] == WALL_CHAR || Character.isDigit(maze[robot.x][robot.y])) {
				// Move robot back since it has walked into a wall or another robot.
				switch (input[1]) {
				case 'U': robots[r].translate(0, 1); break;
				case 'D': robots[r].translate(0, -1); break;
				case 'L': robots[r].translate(1, 0); break;
				case 'R': robots[r].translate(-1, 0); break;
				}
				maze[robot.x][robot.y] = (char)((int)'0' + r);
				continue;
			}

			char newTile = maze[robot.x][robot.y];			
			maze[robot.x][robot.y] = (char)((int)'0' + r);
			
			if (Character.isLetter(oldTile) && Character.isLowerCase(oldTile)) {
				Point hold = holdSwitches.get(String.valueOf(oldTile));
				maze[hold.x][hold.y] = WALL_CHAR;				
			}

			if (newTile == GOAL_CHAR) {
				return steps; // TODO: return steps? What about if they hit goal and then leave it again?
			} else if (Character.isLetter(newTile)) {
				if (Character.isUpperCase(newTile)) {
					Point toggle = toggleSwitches.get(String.valueOf(newTile));
					maze[toggle.x][toggle.y] = maze[toggle.x][toggle.y] == WALL_CHAR ? EMPTY_CHAR : WALL_CHAR;
					
					if (maze[toggle.x][toggle.y] == EMPTY_CHAR) {
						for (int i = 0; i < robots.length; i++) {
							Point rob = robots[i];
							if (rob.x == toggle.x && rob.y == toggle.y)
								maze[toggle.x][toggle.y] = (char)((int)'0' + i);
						}
					}
					
					board.maze[toggle.x][toggle.y] = maze[toggle.x][toggle.y];
				} else { // lower case
					Point hold = holdSwitches.get(String.valueOf(newTile));
					maze[hold.x][hold.y] = EMPTY_CHAR;
					
					if (maze[hold.x][hold.y] == EMPTY_CHAR) {
						for (int i = 0; i < robots.length; i++) {
							Point rob = robots[i];
							if (rob.x == hold.x && rob.y == hold.y)
								maze[hold.x][hold.y] = (char)((int)'0' + i);
						}
					}
					
					board.maze[hold.x][hold.y] = maze[hold.x][hold.y];
				}
			}
		}
		return -1;
	}
	
	public static String generateBoard(int width, int height, int noOfRobots, int noOfToggleSwitches, int noOfHoldSwitches, int minPathLength) {
		MazeRunner mr;
		ArrayList<Pair<Integer, Direction>> res;
		Random r = new Random();
		
		do {
			mr = new MazeRunner();
			
			char[][] maze = new char[width][height];
			for (int i = 0; i < width; i++) {
				for (int j = 0; j < height; j++) {
					maze[i][j] = EMPTY_CHAR;
				}
			}
			mr.goal = new Point(r.nextInt(width), r.nextInt(height));
			maze[mr.goal.x][mr.goal.y] = GOAL_CHAR;			
			
			Point[] robots = new Point[noOfRobots];
			
			for (int i = 0; i < noOfRobots; i++) {
				Point robot;
				do {
					robot = new Point(r.nextInt(width), r.nextInt(height));	
				} while (maze[robot.x][robot.y] != EMPTY_CHAR);
				robots[i] = robot;
				maze[robot.x][robot.y] = (char)((int)'0' + i);
			}
			
			Set<Point> walls = new HashSet<Point>();
			int noOfWalls = r.nextInt(width*height - 2*noOfToggleSwitches - 2*noOfHoldSwitches - 1 - noOfRobots) + noOfToggleSwitches + noOfHoldSwitches;
			for (int i = 0; i < noOfWalls; i++) {				
				Point wall;
				do {
					wall = new Point(r.nextInt(width), r.nextInt(height));
				} while (maze[wall.x][wall.y] != EMPTY_CHAR);
				
				maze[wall.x][wall.y] = WALL_CHAR;
				walls.add(wall);
			}
			
			for (int i = 0; i < noOfToggleSwitches; i++) {
				char c = (char)((int)'A' + i);
				Point wall = (Point)(walls.toArray()[r.nextInt(walls.size())]);
				mr.toggleSwitches.put(String.valueOf(c), wall);
				walls.remove(wall);
				
				Point taylorSwitch;
				do {
					taylorSwitch = new Point(r.nextInt(width), r.nextInt(height));
				} while (maze[taylorSwitch.x][taylorSwitch.y] != EMPTY_CHAR);
				maze[taylorSwitch.x][taylorSwitch.y] = c;
			}
			
			for (int i = 0; i < noOfHoldSwitches; i++) {
				char c = (char)((int)'a' + i);
				Point wall = (Point)(walls.toArray()[r.nextInt(walls.size())]);
				mr.holdSwitches.put(String.valueOf(c), wall);
				walls.remove(wall);
				
				Point taylorSwitch;
				do {
					taylorSwitch = new Point(r.nextInt(width), r.nextInt(height));
				} while (maze[taylorSwitch.x][taylorSwitch.y] != EMPTY_CHAR);
				maze[taylorSwitch.x][taylorSwitch.y] = c;
			}
			
			mr.board = new Board(robots, maze);
			res = mr.BFS();
		} while (res == null || res.size() < minPathLength);
		
		StringBuilder sb = new StringBuilder();
		sb.append(width + " " + height + "\n");
		sb.append(noOfRobots + "\n");
		sb.append(noOfToggleSwitches + " " + noOfHoldSwitches + "\n");
		sb.append(mr.board);
		mr.toggleSwitches.forEach((k, v) -> sb.append(k + " " + v.x + " " + v.y + "\n"));
		mr.holdSwitches.forEach((k, v) -> sb.append(k + " " + v.x + " " + v.y + "\n"));
		String s = sb.toString();
		return s;
	}
	
	public void parse(String s) {
		parse(new BufferedReader(new StringReader(s)));
	}
	
	public void parse() {
		parse(new BufferedReader(new InputStreamReader(System.in)));
	}

	public void parse(BufferedReader br) {
		try {			
			StringTokenizer st = new StringTokenizer(br.readLine());
			int width = Integer.parseInt(st.nextToken());
			int height = Integer.parseInt(st.nextToken());

			int noOfRobots = Integer.parseInt(br.readLine());
			Point[] robots = new Point[noOfRobots];
			
			st = new StringTokenizer(br.readLine());
			int noOfToggleSwitches = Integer.parseInt(st.nextToken());
			int noOfHoldSwitches = Integer.parseInt(st.nextToken());			
			
			char[][] maze = new char[width][height];
			for (int i = 0; i < height; i++) {
				char[] line = br.readLine().toCharArray();
				for (int j = 0; j < width; j++) {
	                if (line[j] == GOAL_CHAR) {
	                    goal = new Point(j, i);
	                } else if (Character.isDigit(line[j])) {
	                    int number = Character.getNumericValue(line[j]);
	                    robots[number] = new Point(j, i);
	                } else if (line[j] != EMPTY_CHAR &&
	                           line[j] != WALL_CHAR &&
	                           !Character.isLetter(line[j])) {
	                    System.err.println("Error: Unknown character '" + line[j] + "' on line " + (i+1) + ", column " + (j+1));
	                }
					
					maze[j][i] = line[j];
				}
			}

			for (int i = 0; i < noOfToggleSwitches; i++) {
				st = new StringTokenizer(br.readLine());
				toggleSwitches.put(st.nextToken(), new Point(Integer.parseInt(st.nextToken()), Integer.parseInt(st.nextToken())));
			}
			
			for (int i = 0; i < noOfHoldSwitches; i++) {
				st = new StringTokenizer(br.readLine());
				holdSwitches.put(st.nextToken(), new Point(Integer.parseInt(st.nextToken()), Integer.parseInt(st.nextToken())));
			}		

	        if (goal == null) {
	            System.err.println("Error: No goal found on board");
	        }
	        
	        this.board = new Board(robots, maze);
			
		} catch (Exception e) {
			e.printStackTrace();
		}
	}
	
	public class Pair<A, B> {
		A fst;
		B snd;
		
		public Pair(A fst, B snd) {
			this.fst = fst;
			this.snd = snd;
		}
	}
	
	public enum Direction {
	    Up, Down, Left, Right;

	    public String toString() {
	        switch (this) {
	        case Up: return "U";
	        case Down: return "D";
	        case Left: return "L";
	        case Right: return "R";
	        default: throw new AssertionError();
	        }
	    }
	}
}


class Board {

	Point[] robots;
	char[][] maze;
	
	public Board(Point[] robots, char[][] maze) {
		this.robots = robots;
		this.maze = maze;
	}
	
	@Override
	public int hashCode() {
		final int prime = 31;
		int result = 1;
		result = prime * result + Arrays.deepHashCode(maze);
		return result;
	}

	@Override
	public boolean equals(Object obj) {
		if (this == obj)
			return true;
		if (obj == null)
			return false;
		if (getClass() != obj.getClass())
			return false;
		Board other = (Board) obj;
		if (!Arrays.deepEquals(maze, other.maze))
			return false;
		if (!Arrays.equals(robots, other.robots))
			return false;
		return true;
	}

	public String toString() {
		StringBuilder sb = new StringBuilder();
		for (int i = 0; i < maze[0].length; i++) {
			for (int j = 0; j < maze.length; j++) {					
				sb.append(maze[j][i]);
			}
			sb.append('\n');
		}
		return sb.toString();
	}
}


