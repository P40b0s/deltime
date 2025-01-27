### Консольная утилита для запланированного удаления файлов
#### Использование с файлом конфигурации config.toml:  
``` toml
[[tasks]]
#абсолютный путь к файлу
file_path = "/hard/xar/projects/tests/1"
#интервал удаления в минутах после запуска программы
del_time_interval = 2
repeat = false

[[tasks]]
file_path = "/hard/xar/projects/tests/2"
#или указать нужную дату со временем в формате
del_time = "2025-01-27T16:39:50"
repeat = false

[[tasks]]
file_path = "/hard/xar/projects/tests/3"
del_time = "2025-01-27T16:42:50"
repeat = false

[[tasks]]
file_path = "/hard/xar/projects/tests/4"
del_time_interval = 2
#повторять удаление файла через N минут, работает только со свойством del_time_interval
repeat = true
```
При запуске программы файл конфигурации будет удален   
#### Использование с аргументами при запуске  
`-f` абсолютный путь к файлу  
`-i` интервал удаления в минутах (опционально, либо параметр -d)  
`-d` точная дата удаления в формате "2025-01-27T16:42:50"  
`-r` повторять процесс удаления бесконечно, если установлен флаг -i  
`-f "/hard/x/projects/tests/1" -d "2025-01-27T11:52:00"`  
`-f "/hard/x/projects/tests/1" -i 10`  
`-f "/hard/x/projects/tests/1" -i 10 -r`  
[Linux](https://github.com/P40b0s/deltime/releases/download/v0.1.0/deltime)  
[Windows](https://github.com/P40b0s/deltime/releases/download/v0.1.0/deltime.exe)
