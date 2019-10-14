# d5 — The DIY Dynamic DNS
*The simple, Unix-philosophy tool to retrieve the your home network's IP address
remotely*

If you want to know the IP address of the computer you're *currently* using,
there are [a](https://github.com/georgyo/ifconfig.io)
[lot](https://github.com/mehulved/ifconfig.me/blob/master/ifconfig.me)
[of](https://github.com/pmarques/ifconfig.me)
[tools](https://github.com/missdeer/ifconfig) you can choose from.

If, however, you want to know the IP address of some *other* computer—say, you
want to know the IP address of your home computer while you're traveling—your
options are much more limited.  That's where d5 comes in.  With d5, you set a
simple script on your home computer to update the IP address associated with a
username—password pair.  Once that's done, you can access the current value of
that IP address from anywhere in the world, using that same username–password
pair.

For example, if you want to use the public d5 server I host, you could run this
cron job on your home computer (replacing `USER` and `PASSWORD` with your actual
values, of course):

```shell
*/5 * * * * curl -u USERNAME:PASSWORD https://ip.codesections.com -X POST
```

You can substitute any other timing method (systemd timers, emacs timers,
runwhen scripts, etc).  The only important point is that it run the `curl -u
USERNAME:PASSWORD https://ip.codesections.com -X POST` command on a regular
schedule.

Once you've done that, you can access the most-recently updated IP address with
the following curl command:

`curl -u USERNAME:PASSWORD https://ip.codesections.com`

If you want to automate the process of using d5, it is simple to do so with
shell scripts/aliases.  For example, I use the following Bash alias:

`alias ssh-home='ssh $(curl $USER https://ip.codesections.com) -p $SSH_PUBLIC_PORT`

(Omitting the `:PASSWORD` portion from the alias causes curl to prompt me for
the password on each use.  If you don't want the security/hassle of being
prompted for your password, you could store the password as plain text or using
your preferred credential storage method.)

If you are happy using the public d5 server at https://ip.codesections.com, then
this is all you need to know.  If you would like to self-host d5, then read on.

## Self hosting d5

If you prefer not to use the ip.codesections.com server, you can also host d5
yourself.  (Of course, you will need to host d5 somewhere you can access
remotely.)

### Installing d5

Because d5 is distributed as a statically linked binary, installing it on any
x86 Linux distribution is as simple as downloading the latest release and 
making it executable (`chmod +x ./d5`).

d5 is designed to run in server environments, and thus does not currently
provide binaries for macOS, Windows, or ARM (e.g., Raspberry Pi) computers.  If
you would like a precompiled binary for one of those platforms, please open an
issue.  In the meantime, you can build d5 from source using `cargo`, the Rust
package manager.

### Configuring and Running d5

Run d5 by invoking it from the command line (`./d5`).  You can configure it
using environmental variables; d5 currently supports the following variables:

* `PORT`: the port on which to run d5 (if unspecified, defaults to `3030`)
* `HOST`: the host address on which to run d5 (if unspecified, defaults to
  `127.0.0.1`).  `HOST` may be specified as an IPv4 address or a string (e.g.,
  `localhost`).
* `KEY`: If set, enables **single-user mode**, described below, and sets the 
   `username:password` key for single-user mode.
   
By default, d5 is in **multi-user mode**.  In this mode, d5 allows anyone to
store IP addresses and retrieve them with the associated username–password pair.
If you provide a `KEY` environmental variable, d5 will run is **single-user
mode**, and will *only* allow a single IP address to be stored with the
username–password pair provided via the `KEY` environmental variable.  When
setting the `KEY` variable, you must provide the username and password in the
same format curl uses: separated by a colon (`username:password`).

### Using d5 with a Reverse Proxy (e.g., Nginx)

Although you *could* directly expose d5 to the public Internet, a more common
deployment strategy would be to place d5 behind a reverse proxy, such as Nginx
or Traefik.  If you do so, you will need to configure your reverse proxy to
forward on the incoming IP address using either the `remote_addr` or
`x-forwarded-for` header.  (d5 reads these headers to learn the relevant IP 
address).  For example, the following is a minimal Nginx configuration block
for a server located at `d5.codesections.com`:

```nginx
server {
    server_name ip.codesections.com;
    location / {
        proxy_pass http://localhost:3030;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}```

Depending on your desired security, you will almost certainly want to set up
https (e.g., with LetsEncrypt) and may additionally want to set rate limits.


## Using the IP Address Returned by d5

As mentioned above, d5's *primary* use case is returning a computer's current IP
address to provide a target for `ssh` or similar commands.  However, in the long
tradition of Unix tools, d5 provides simple, machine- and human-readable output
and can be easily combined with other tools to solve more complex problems.

For example, if you want a *full* dynamic DNS solution (rather than a DIY
version), you can pass the new IP address to [DNS Lexicon](https://github.com/AnalogJ/lexicon)
and let it handle updating the DNS record associated with a particular URL.

## Goals

d5 should be: 
* **simple/minimalist** — d5 is solving a simple problem: you don't know the 
  IP address of a computer and would like to.  This should not be hard.  d5's 
  source code is currently a single file with 73 lines of code; I can't think
  of any reason it should ever exceed 100 lines of code.
* **lightweight** — consistent with the above, d5 should not use many CPU 
  cycles or much RAM to solve such a simple problem.
  * **private and reasonably secure** — d5 should never log IP or user information
  and, to the extent that it does not detract from the first two goals, should
  be as secure as practicable.
  
## Non-Goals

d5 is *not* attempting to:
* provide information *other than* IP address (useragent, etc.).  Use ifconfig.me instead.
* provide a full (non-DIY) dynamic DNS solution.  Use [DDclient](https://sourceforge.net/p/ddclient/wiki/Home/) duckdns/a similar service instead.  (Or use d5 + DNS Lexicon, as described above.)


## FAQ

#### Why should I use d5 instead of a full dynamic DNS solution like DDclient?

Maybe you shouldn't!  DDclient/similar software does two things: 1) it monitors your current IP 
address and 2) it updates DNS settings to point a human-readable URL to the new address.  If 
you need both of these features—that is, if you want a human-readable URL set dynamically, then
it *might* make sense to use DDclient.   <small>(Though I think there's still a Unix-philosophy/
separation of concerns case to be made for splitting the two tasks into separate programs and 
using d5 + DNS Lexicon.)</small>

But, many times, you *don't* need a true dynamic DNS with a human-readable URL—you just need
a way to connect to a computer regardless of its changing IP address.  If you don't need that
extra functionality, then taking on the code complexity of something like DDclient is excessive.
For example, DDclient is over 4,000 lines of Perl; d5 is under 100 lines of Rust code.  4,000
lines of code provides a much larger surface area for bugs.

#### Why should I use d5 instead of DuckDNS or a similar service?

Privacy and control.  I have nothing against DuckDNS or any service—DuckDNS's [privacy policy](https://www.duckdns.org/pp.jsp) seems pretty decent, as far as such things go.  But they have a privacy policy because they *do* collect personal data—they have to, to provide the service they do.  d5 does not store
your data in any way and, if you don't trust the version running at ip.codesections.com, you can 
trivially self-host your own copy.

#### Why should I use d5 instead of selfhosting ifconfig.io or something similar?

Simplicity.  Tools like ifconfg do both too much and too little.  They do too much in that they a large
amount of information in addition to IP address; you don't need this information to connect 
remotely, and collecting it just makes the code more complicated.  They do too little in that
they tell you about the IP address of the *current* computer and don't let you retrieve the IP
address of a different computer.  This means that, when using ifconfig, you need to *first* retrieve the IP address for your computer and *then* find a way to send that IP address elsewhere.  This involves 
multiple round-trips and more complexity than is justified by the simple task.

#### How secure is d5?

d5 provides decent security, but not excellent.  d3 does not store IP address or username–password
pairs on disk and thus a compromise of d3 servers cannot leak any of that data.  However, because d3 uses [basic authentication](https://developer.mozilla.org/en-US/docs/Web/HTTP/Authentication#Basic_authentication_scheme), username–password pairs are transmitted in plaintext (aside from the encryption provided by HTTPS).  Thus, anyone who *thoroughly* compromised a d3 server would be in a position to intercept IP 
addresses and username–password pairs.  Additionally, d3 does not itself implement rate limiting 
(though it's easy to so at the reverse proxy level).  This means that, depending on proxy configuration,
weak username–password pairs could be vulnerable to brute forcing. 

#### Shouldn't d5 store IP addresses in a database like Postgres or Redis rather than keeping them in memory?

No.  One of the d5's primary goals is to be as simple as possible and relying on a separate database would be the exact opposite of "simple".

#### Ok, but shouldn't d5 at least store IP addresses in a text file?  Keeping them in memory just seems … fragile. 

That was my first thought too (and the initial implementation for d5), but two considerations changed my mind.  First, storing the passwords would both require hashing them (increasing complexity) and would create the possibility of an attacker gaining access to the hashed passwords (decreasing security).

Second, and more importantly, I realized that persisting the IP addresses is entirely unnecessary.  The normal reason to persist data to the hard drive is to prevent data loss in the case of a program crash or shutdown.  But the entire idea behind d5 is that the IP address is constantly subject to change and is being updated every few minutes.  So, if d5 crashes, no meaningful data is lost—within 5 minutes, all IP 
addresses will be added back to the system.

#### Is it really fair to call d5 "DIY Dynamic DNS"?  It doesn't create any DNS entries.

That's a fair point, and what I am attempting to get at by calling it "DIY".  If you want to call it
"self-hosted remote IP address retrieval", I won't argue with you.  But that doesn't roll of the tongue 
quite as well.

##### Why the name d5?  I notice that "DIY Dynamic DNS" only has three Ds.

Well, mostly because I want to avoid name collision with [d3.js](https://d3js.org/), the JavaScript
data-visualization library.  But if the two missing Ds bother you, you can think of this project
as "Daniel's DIY Dynamic DNS for Dimwits" (with the "for Dimwits" part referring to the obsessive focus on simplicity).

