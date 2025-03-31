# development_initializer
This is a currently small and simple project initializer.

Have you ever had to develop applications requiring many commands of startup?
Have you ever had the to input *gasp* multiple commands to get you ready to code?
Have you ever had the deeply unfortunate experience of having to program web applications requiring you to both bootup the local server AND open up your ide?

Well no more.

Now you can simply specify commands in a .toml project configuration file, run a command and you are off to the races.

As initially stated this project is still in its infancy, and as I am in my rust knowledge infancy its a really unelegant solution.

Either way, I can now simply ```goto <project-name>``` or ```goto <project-name-alias>``` instead of having to type 2 sentences.

## Usage
I like the command to be goto(its great because of past debugging trauma), but you can easily change that by just renaming the goto file to whatever command you prefer.

then simply update your config.toml(full path in installation section) to look like the following
```
[project_aliases]
//(don't worry about adding .toml in the <Name of project config> we append it in the program)
<Name of project config> = "alias1, alias2, alias3"
```

then in that same config directory there will be a ```/projects/```. Here is where you would create your <Name of project config>.toml file which would look something like. Your first command should likely always be a cd to the directory so that the commands execute in the right place.

```
commands = [
    "command1",
    "command2",
    "command3"
]
```

Example:

config.toml:
```
[project_aliases]
Program = "project, proj, p"
```

/projects/Program.toml:
```
commands = [
    "cd Repos/Program/",
    "docker-compose up -d",
    "nvim ."
]
```

which then can simply be ran by typing ```goto project```, ```goto proj```, or ```goto p```

### Future
- I'll be adding a flag that creates these entries for you but for now this is what we have.
- I am also going to be refactoring so it gives you a brand new interactive shell instead of modifying the parent shell. This will probably be changeable in the config to this version though.
- Let me know if there are any other useful and **simple** features I might add. I don't really want to make this compatible with fridge OS or add the ability to goto a real life location. I might add stuff like multiplexing and tmux initialization though who knows.


## Installation
Simply
``` cargo build --release ```

and move the compiled executable to ```/etc/development_initializer/```

then move the ```src/goto``` to ```/usr/local/bin/``` and chmod +x it so it can be ran.

Lastly, you'll need a ```~/.config/development_initializer/config.toml``` and a ```~/.config/development_initializer/projects/```

I will probably introduce an an initialization or installation command at some point, but today is not that day.