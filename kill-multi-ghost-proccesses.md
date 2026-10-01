ps aux | grep -Ei '[V]isual Studio Code|[C]ode Helper|[c]ode'
ps -o pid,ppid,stat,etime,command -p \\n44269,21392,46915,46821,45926,24279,23837,22894
kill -TERM 21392 44269
ps -o pid,ppid,stat,etime,command -p \\n44269,21392,46915,46821,45926,24279,23837,22894
kill -TERM 22894 45926
ps -o pid,ppid,stat,etime,command -p \\n44269,21392,46915,46821,45926,24279,23837,22894l


