# git-stats-web
A web app for generating reports from your git commit history.

<img src="https://github.com/Some1and2-XC/git-stats/blob/main/examples/server.png" />

## Features

### Easily Generate Weekly Work Reports ([example](https://github.com/Some1and2-XC/git-stats/blob/main/examples/may_12-18_2024.pdf))
Have you ever needed to make a report of what you have been working on for any reason as a software dev? Needing to use programs to manually keep track of what you have been doing can be an arduous task. Especially since you already have been doing exactly that but in a way that's more reasonable: using git. What this program does is allows you to see all the work you have done but in a more legible format than the one that is default for storing your git files. If you go in the list view you can even use your browsers print functionality to make a report of all the things you have done in the week.

### Starting Point Projection
When you make commits, generally the workflow is you write some code, then commit your changes. Because of this when you start working isn't actually tracked. Wouldn't it be nice if your fancy calendar generator could make some assumptions about when you started so that you get credit for all the work that you did? This program takes the total amount of lines added/removed and keeps track of the amount of time it takes on average for both of these metrics. This is so that every commit is counted.

### Support for local files
If you work for the NSA and you don't want your state of the art kernel level 0days on some public repo but also want to get reports, T-DY supports running on local repos! By using the --allow-local option you can use `file://` url encoding and generate reports from your local file system (this is disabled on the web version.)

## In Development!
 - Adding more fine grained options for get requests like the ability to modify how long apart commits can happen while still being counted as one session.
 - Allow more control over the API client side, such as get <i>n</i> commits or get commits until <i>dd-mm-yyyy</i> (instead of just sending everything and hoping for the best.)
 - Adding a progress bar that hooks into the fetch callback using WebSockets.

## Shoutouts
<i>A series of articles that helped massively for the initial version of this project was from a [dev.to](https://dev.to/calebsander/git-internals-part-2-packfiles-1jg8) user calebsander.</i>
