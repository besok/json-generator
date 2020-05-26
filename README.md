## Json generator
The console utility to generate JSON items according to the provided example composing JSON body 
and a set of functions defining the logic to generate new items. 
The utility allows delivering the generated JSON to different sources such as HTTP server, folder or file

### Generators
The function can be added to json file above the proper field with /* */ distinction as foolows:
```
{
/* generator(args)*/
"field" : "value"
}
```

#### List of generators:
| Generator | Arguments | Description | Example |
|----------------------|--------------------------------------------|----------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------|
| sequence | starting point | the sequentially-increase row of numbers (1,2,3,4 ...) | sequence(10)  |
| random_str | size of row | the row composed of random letters and numbers, predefined length | random_str(10) |
| random_int | low bound and high bound | the random number lying in predefined bounds | random_int(1,100) |
| random_str_from_file | path to file, delimiter(optional) | the list of string pulled off the predefined file note: delimiter can be omitted and the default delimiter(,) will be used | random_str_from_file(\home\user\json) random_str_from_file(\home\user\json,;) random_str_from_file(\home\user\json,\n) |
| random_int_from_file | path to file, delimiter(optional)  | list of numbers pulled off the predefined file note: delimiter can be omitted and the default delimiter(,) will be used  | random_int_from_file(c:\\user\json)  |
| random_str_from_list | list of values | the list of string | random_str_from_list(a,b,c,d) |
| random_int_from_list | list of values | list of numbers | random_int_from_list(1,2,3,4,5) |
| uuid |  | generated uuid  | uuid() |
| current_date_time | format | the current date and time. By default can be ommited  and '%Y-%m-%d %H:%M:%S' will be used | currnet_date_time(%Y-%m-%d) |
| array | number of elements, generator for elements | the generator to get the array filled. | array(10,random_int(1,10)) |


### Command line example

```
json-generator.exe  -f "file path" -r 10  --pretty --print --to-folder folder--to-curl '-X POST ip'
```    

#### Command line Arguments
| Short | Long  | Description                                                                                                 | Example                                                               |
|----------|-----------|-------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------|
| b        | json-body | the text reprensting the json body                                                                          | --json-body \| -b '{"k":"v"}'                                         |
| f        | json-file | the path to file including the json                                                                         | --json-file \| -f c:\\folder\json.json                                |
| r        | repeat    | the number of repetetions of generating                                                                     | --repeat \| -r 10                                                     |
|          | pretty    | inserts formatting symbols in proper places                                                                 | --pretty                                                              |
|          | print     | prints logs                                                                                                 | --print                                                               |
|          | to-cmd    | show json in console(by default if outputs array is empty)                                                  | --to-cmd                                                              |
|          | to-file   | append generated jsons to file. If the file does not exist.  it creates a new one. The folder should exist. | --to-file c:\\folder\jsons.json                                       |
|          | to-folder | creates new files and place it to the selected folder.  It creates folder if it not exists.                 | --to-file c:\\folder                                                  |
|          | to-curl   | sends jsons to the server using curl for that. In fact,  the -d will be added.                              | --to-curl '-H "Content-Type:application/json" -X POST 127.0.0.1:7878' |
| -h       | --help    | infroamtion about commands                                                                                  | -h \| --help                                                          |
| -V       | --version | version                                                                                                     | -V \| --version                                                       |
 
#### Json example

```
{
  "person": {
    /* sequence(1) */
    "id": 1,
    /* current_date_time() */
    "update_tm": "",
    /*random_str(10)*/
    "name": "Eli\"za\"beth",
    /* random_str_from_list(a,b,c,d) */
    "surname": "EVa",
    /*random_int(20,40)*/
    "age": 10,
    /*array(3,sequence(1))*/
    "children_ids": [
      3,
      6
    ],
    "address": {
      /*random_str(10)*/
      "street": "Grip",
      /*random_int(1,100)*/
      "house": 10,
      /* random_str_from_file(C:\projects\json-generator\jsons\cities.txt,\r\n)*/
      "city": "Berlin"
    }
  }
}
```