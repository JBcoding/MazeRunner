using System;
using System.Linq;
using System.Collections;
using System.Text;
using System.Diagnostics;
using System.IO;
using System.Collections.Generic;

class MapGenerator
{
    public static void Main(string[] args)
    {
        var generator = new MapGenerator();

        int seed = 0;
        int size = 40;

        while (true)
        {
            generator.Generate(seed, size, 2, 0.3);

            if (generator.Validate(10))
            {
                Console.WriteLine("Accepted!");
                generator.Save("new3/2-" + size + "-Test");
                size++;
            }
            else
                Console.WriteLine("Not accepted!");

            seed++;
        }
        
        Console.ReadLine();
    }

    private int N, R;
    private char[][] map;
    private Random Rnd;
    private string[] solution;
    private Tuple<int, int> goalPos;
    private Tuple<int, int>[] robotPos;

    public void Generate(int seed, int N, int R, double wallFactor)
    {
        Rnd = new Random(seed);

        this.R = R;
        this.N = N;

        map = Enumerable.Range(0, N).Select(i => Enumerable.Repeat(' ', N).ToArray()).ToArray();

        for (int i = 0; i < (int)(N * N * wallFactor); i++)
        {
            var pos = getFreePos();
            map[pos.Item1][pos.Item2] = '#';
        }


        robotPos = new Tuple<int, int>[R];

        for (int i = 0; i < R; i++)
        {
            var pos = getFreePos();
            map[pos.Item1][pos.Item2] = (char)('0' + i);
            robotPos[i] = pos;
        }

        goalPos = getFreePos();
        map[goalPos.Item1][goalPos.Item2] = 'G';
    }

    private Tuple<int, int> getFreePos()
    {
        while (true)
        {
            int x = Rnd.Next(N), y = Rnd.Next(N);

            if (map[y][x] == ' ') return Tuple.Create(x, y);
        }
    }

    public void Save(string name)
    {
        File.WriteAllText(name + ".in", GetMapString());
        File.WriteAllText(name + ".out", String.Join("\n", solution));
    }

    public string GetMapString()
    {
        var sb = new StringBuilder();

        sb.AppendLine(N.ToString());
        sb.AppendLine(R.ToString());

        foreach (var line in map)
        {
            sb.AppendLine(new string(line));
        }

        return sb.ToString();
    }

    private bool canReachGoal()
    {
        var visited = new bool[N, N];

        var queue = new Queue<Tuple<int, int>>();

        Action<int, int> add = (y, x) => { if (x < N && y < N && x >= 0 && y >= 0 && !visited[y, x] && map[y][x] != '#') { visited[y, x] = true; queue.Enqueue(Tuple.Create(y, x)); } };

        add(goalPos.Item1, goalPos.Item2);

        while (queue.Count > 0)
        {
            var cur = queue.Dequeue();

            add(cur.Item1 + 1, cur.Item2);
            add(cur.Item1 - 1, cur.Item2);
            add(cur.Item1, cur.Item2 + 1);
            add(cur.Item1, cur.Item2 - 1);
        }

        foreach (var pos in robotPos)
        {
            if (!visited[pos.Item1, pos.Item2])
                return false;
        }

        return true;
    }

    public bool Validate(int timelimit)
    {
        if (!canReachGoal())
        {
            Console.WriteLine("Can't reach goal");
            return false;
        }

        ProcessStartInfo startInfo = new ProcessStartInfo();
        startInfo.FileName = "solver.exe";
        startInfo.RedirectStandardInput = true;
        startInfo.RedirectStandardOutput = true;
        startInfo.RedirectStandardError = true;
        startInfo.UseShellExecute = false;

        var proc = Process.Start(startInfo);
        proc.StandardInput.Write(GetMapString());
        proc.StandardInput.Flush();
        proc.StandardInput.Close();
        DateTime dt = DateTime.Now;
        proc.WaitForExit(timelimit * 1000);
        int spentTime = (int)(DateTime.Now - dt).TotalSeconds;

        if (!proc.HasExited)
        {
            Console.WriteLine(GetMapString());
            proc.Kill();
            return false;
        }

        solution = proc.StandardOutput.ReadToEnd().Split('\n').Where(a => !String.IsNullOrEmpty(a)).ToArray();
        int involvedRobots = solution.Select(s => s[0]).Distinct().Count();

        Console.WriteLine("Length: " + solution.Length + " Robots: " + involvedRobots);
        Console.WriteLine(String.Join(", ", solution));

        return involvedRobots >= R - 0 && spentTime > -1;
    }


}