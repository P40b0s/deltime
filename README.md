### Консольная утилита для запланированного удаления файлов
![screen](https://github.com/user-attachments/assets/25787f71-0d22-44f5-a284-7fcaf6a327d1)

#### Использование с файлом конфигурации config.toml:  
``` toml
[[tasks]]
path = "/tests/1"
interval = 1
repeat = "once"
visible = true

[[tasks]]
path = "/tests/2"
date = "2025-02-15T21:33:44"
repeat = "once"
visible = true

[[tasks]]
path = "/tests/3"
date = "2025-02-15T21:36:44"
repeat = "monthly"
visible = true

[[tasks]]
path = "/tests/4"
date = "2025-02-15T21:33:44"
repeat = "dialy"
visible = false

[[tasks]]
path = "/tests/5"
mask = "*.delme"
interval = 3
repeat = "forever"
visible = true

[[tasks]]
path = "/tests/not_exists"
interval = 1
repeat = "once"
visible = true

[[tasks]]
path = "/tests/expired"
date = "2025-02-15T21:27:44"
repeat = "once"
visible = true
```

`path` - полный путь к файлу или директории  
`mask` - необязательный параметр, работает только если для обработки указана директория, примеры: \*.txt, file\*.txt, file\*  
`interval` - альтернативный параметр с параметром `date`, указывает интервал таймера в минутах  
`date` - альтернативный параметр с параметром `interval`, указывает точное время  
`repeat` - стратегия повтора задачи  
- `once` - задача выполняется один раз и потом завершается  
- `dialy`|`forever` - для `interval` задача будет запускаться бесконечно при каждом обнулении таймера, для `date` задача будет запускаться ежедневно в указанное время  
- `monthly` - только для `date`, задача будет запускаться ежемесячно в указанное время и дату  

`visible` отображение дополнительной информации рядом с прогрессбаром  

При запуске программы будет попытка считать файл конфигурации из директории запуска, если файл не обнаружен программа перейдет в режим ожидания, файл конфигурации может быть автоматически загружен с флеш накопителя, если он присутсвует на флеш накопителе программа автоматически его считает и добавит задачи в список. 

