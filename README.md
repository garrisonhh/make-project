# make-project

I make a lot of small projects, and I use a lot of different tools. Instead of wasting the same hour or so every time I make a new project figuring out all of the configuration I want or need, this tool can be used to automate away that process.

the aspirational goal is that starting any project can look like:
```bash
make-project [OPTIONS] $PROJECT_FOLDER
cd $PROJECT_FOLDER
nix develop
```
and I have a complete dev environment that I only have to modify slightly if at all.