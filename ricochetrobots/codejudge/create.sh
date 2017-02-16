NAME=$1
COMPETITION=$2
TIMELIMIT=5
MEMORYLIMIT=500

mkdir temp
mkdir temp/judge
cp judge/Judge$COMPETITION.cs temp/judge/Judge.cs
cp -R views temp/views
cp config$COMPETITION.xml temp/config.xml
cp ../boards/$NAME/* temp/
cd temp/
for f in *.in
do
	filename="${f%.*}"
	echo "$TIMELIMIT $MEMORYLIMIT" > "$filename.args"
done
zip -r ../$NAME.zip *
cd ../
rm -R temp