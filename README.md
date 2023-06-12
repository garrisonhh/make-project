# make-project

I make a lot of small projects, and I use a lot of different tools. Instead of wasting the same hour or so every time I make a new project figuring out all of the configuration I want or need, this tool can be used to automate away that process.

## how do I build your stupid project

requires nix.

```bash
git clone https://github.com/garrisonhh/make-project.git
cd make-project
nix build .
```

## usage

```bash
make-project [OPTIONS] $PROJECT_FOLDER
cd $PROJECT_FOLDER
nix develop
```

further development of this project will result in more features for making the output `$PROJECT_FOLDER` exactly what I would want.