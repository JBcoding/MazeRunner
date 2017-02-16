using System;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using System.Web;

class Fixer
{
    public static void Main(string[] args)
    {
        foreach (var file in Directory.GetFiles(".").Where(c => c.Contains("Test")))
        {
			File.Move(file, file.Replace("Test", "Random"));
            /*string input = File.ReadAllText(file), output = File.ReadAllText(file.Replace(".in", ".out"));
            string game = String.Format(@"{{""board"":""{0}"",""moves"":""{1}"",""pastMoves"":""0""}}", input.Replace("\r", "").Replace("\n", "\\n"), output.Replace("\r", "").Replace("\n", "\\n"));
            game = HttpUtility.UrlEncode(game).Replace("+", "%20");

            File.AppendAllText("maps.txt", Path.GetFileName(file) + ": ");
            File.AppendAllText("maps.txt", "https://hwv.dk/rr/#" + game + "\r\n\r\n");*/
        }

        Console.ReadLine();

    }
}