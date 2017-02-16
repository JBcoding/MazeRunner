using System;
using System.IO;
using System.Linq;

public class Judge
{
    private static char WALL = '#';
    private static char FREE = ' ';
    private static char GOAL = 'G';
    private static char[] directions = new char[] { 'U', 'R', 'D', 'L' };
    private static char[][] map;
    private static int N, R;
    private static int[][] robots;

    public static void Main(string[] args)
    {
        // Read the output from the testee
        // WARNING: All data from this file should be handled
        // carefully as their programs may have outputted weird stuff
        string testeeOutput = File.ReadAllText("output");

        // Split the output in tokens
        string[] testeeTokens = testeeOutput.Split(' ', '\t', '\n', '\r')
            .Where(s => !String.IsNullOrEmpty(s)).ToArray();

        // Read the map from stdin
        N = int.Parse(Console.ReadLine());
        R = int.Parse(Console.ReadLine());

        map = Enumerable.Range(0, N).Select(a => Console.ReadLine().ToCharArray()).ToArray();

        robots = new int[R][];
        int[] goal = null;

        for (int row = 0; row < N; row++)
        {
            for (int column = 0; column < N; column++)
            {
                if (map[row][column] >= '0' && map[row][column] <= '9') 
                {
                    robots[map[row][column] - '0'] = new int[] { row, column };
                    map[row][column] = FREE;
                }
                else if (map[row][column] == GOAL)
                { 
                    goal = new int[] { row, column };
                    map[row][column] = FREE;
                }
            }
        }

        // Simulate their robot move sequence
        foreach (var token in testeeTokens)
        {
            if (token.Length != 2 || !(token[0] >= '0' && token[0] <= '9') || !directions.Contains(token[1]))
            {
                fail("Unrecognized token \"" + sanitize(token) + "\"");
                return;
            }

            int robot = token[0] - '0';

            if (robot >= R)
            {
                fail("You tried to move robot #" + robot + ", but there are only " + R + " robots");
                return;
            }

            char direction = token[1];

            move(robots[robot], direction);
        }

        // Check goal is reached
        if (robots[0][0] != goal[0] || robots[0][1] != goal[1])
        {
            fail("Robot #0 did not end up in goal field");
            return;
        }

        // They solved the map, gj!
        string reason = "Correct [" + testeeTokens.Length + " move" + (testeeTokens.Length != 1 ? "s" : "") + "]";

        // Read expected output file (TestXX.out) to see if we have a better solution
        // (we can make all the assumptions we like about this file since we made it ourselfs)
        if (File.Exists("expected"))
        {
            string[] expectedTokens = File.ReadAllText("expected").Split(' ', '\t', '\n', '\r')
                .Where(s => !String.IsNullOrEmpty(s)).ToArray();

            if (expectedTokens.Length < testeeTokens.Length)
            {
                reason += " (can be solved using only " + expectedTokens.Length + " moves)";
            }
        }

        Console.WriteLine("RESULT CORRECT");
        Console.WriteLine("TEXT " + reason);
    }

    private static void move(int[] pos, char dir)
    {
        while (true)
        {
            int a = dir == 'U' || dir == 'D' ? 0 : 1;
            int step = dir == 'L' || dir == 'U' ? -1 : 1;

            pos[a] += step;

            if (pos[a] < 0 || pos[a] >= N || map[pos[0]][pos[1]] == WALL || robots.Any(r => r != pos && r[0] == pos[0] && r[1] == pos[1]))
            {
                pos[a] -= step;
                return;
            }
        }
    }

    private static string sanitize(string text)
    {
        if (text.Length > 10)
            return text.Remove(10) + "...";

        return text;
    }

    private static void fail(string reason)
    {
        Console.WriteLine("RESULT WRONG");
        Console.WriteLine("TEXT " + reason);
    }
}