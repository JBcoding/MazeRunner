from enum import Enum

class Point:
	def __init__(self, x, y):
		self.x = x
		self.y = y
		
	def __repr__(self):
		return self.__str__();
		
	def __str__(self):
		return "Point: { x: " + str(self.x) + " y: " + str(self.y) + " }"
		
class Direction(Enum):
	Up, Down, Left, Right = range(4)
	
	def __str__(self):
		return self.name[0]
		
EMPTY_CHAR = ' '
WALL_CHAR = '#'
GOAL_CHAR = '!'

goal = None
toggle_switches = {}
hold_switches = {}
maze = []

def compute_solution():
	# TODO: Implement really clever stuff here!
	# For now - just output a couple of moves:
	print "0" + str(Direction.Left)
	print "0" + str(Direction.Down)
	
	print "Goal: "
	print goal
	print "Robots: "
	print robots
	print "Toggle switches:"
	print toggle_switches
	print "Hold switches:"
	print hold_switches
	
	for i in maze:
		print i

def parse():
	dims = raw_input().split(' ')
	width = int(dims[0])
	height = int(dims[1])
	
	no_of_Robots = int(raw_input())
	global robots
	robots = [None] * no_of_Robots
		
	switches = raw_input().split(' ')
	no_of_toggle_switches = int(switches[0])
	no_of_hold_switches = int(switches[1])
	
	for i in range(height):
		line = raw_input()
		row = []
		for j in range(width):
			if line[j] == GOAL_CHAR:
				goal = Point(j, i)
			elif line[j].isdigit():
				robots[int(line[j])] = Point(j, i)
			elif line[j] != EMPTY_CHAR and line[j] != WALL_CHAR and not line[j].isalpha():
				print "Error: Unknown character '" + line[j] + "' on line " + (i+1) + ", column " + (j+1)
			
			row.append(line[j])
			
		maze.append(row)
			
	for i in range(no_of_toggle_switches):
		switch = raw_input().split(' ')
		toggle_switches[switch[0]] = Point(int(switch[1]), int(switch[2]))
			
	for i in range(no_of_hold_switches):
		switch = raw_input().split(' ')
		hold_switches[switch[0]] = Point(int(switch[1]), int(switch[2]))
		
	if goal is None:
		print "Error: No goal found on board"

parse()
compute_solution()