import java.awt.Point;
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.util.Arrays;
import java.util.HashMap;
import java.util.Map;
import java.util.StringTokenizer;

public class MazeRunner {

	final char EMPTY_CHAR = ' ';
	final char WALL_CHAR = '#';
	final char GOAL_CHAR = '!';

	private Point goal;
	private Point[] robots;
	 // Switches and the walls, which they affect
	private Map<String, Point> toggleSwitches = new HashMap<String, Point>();
	private Map<String, Point> holdSwitches = new HashMap<String, Point>();
	private char[][] maze;
	
	public static void main(String[] args) {
		MazeRunner mr = new MazeRunner();
		mr.parse();
		mr.computeSolution();
	}
	
	public void computeSolution() {
        // TODO: Implement really clever stuff here!
        // For now - just output a couple of moves:
        System.out.println("0"+Direction.Left);
        System.out.println("0"+Direction.Down);        
        
//        System.out.println("Goal: " + goal);
//        System.out.println("Robots: " + Arrays.toString(robots));
//        
//        System.out.println("Toggle switches: " + toggleSwitches);
//        System.out.println("Hold switches: " + holdSwitches);
//
//        for (int i = 0; i < maze.length; i++) {
//        	System.out.println(Arrays.toString(maze[i]));
//        }
	}

	public void parse() {
		try {	
			BufferedReader br = new BufferedReader(new InputStreamReader(System.in));
			
			StringTokenizer st = new StringTokenizer(br.readLine());
			int width = Integer.parseInt(st.nextToken());
			int height = Integer.parseInt(st.nextToken());

			int noOfRobots = Integer.parseInt(br.readLine());
			robots = new Point[noOfRobots];
			
			st = new StringTokenizer(br.readLine());
			int noOfToggleSwitches = Integer.parseInt(st.nextToken());
			int noOfHoldSwitches = Integer.parseInt(st.nextToken());			
			
			maze = new char[height][width];
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
					
					maze[i][j] = line[j];
				}
			}

			for (int i = 0; i < noOfToggleSwitches; i++) {
				st = new StringTokenizer(br.readLine());
				toggleSwitches.put(st.nextToken(),
						new Point(Integer.parseInt(st.nextToken()), Integer.parseInt(st.nextToken())));
			}
			for (int i = 0; i < noOfHoldSwitches; i++) {
				st = new StringTokenizer(br.readLine());
				holdSwitches.put(st.nextToken(),
						new Point(Integer.parseInt(st.nextToken()), Integer.parseInt(st.nextToken())));
			}		

	        if (goal == null) {
	            System.err.println("Error: No goal found on board");
	        }
			
		} catch (Exception e) {
			e.printStackTrace();
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



