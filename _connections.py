import os

directories = [

    
"/home/southatoms/Escritorio/litevibrance_backend/backend_vibrance/src"

]

file_extensions = ['rs',]
#file_extensions = ['.js', '.tsx', '.ts', '.jsx', '.html', '.css', '.py', '.java','rs']

output_file = '_copnnections.txt'

def is_code_file(file):
    return any(file.endswith(ext) for ext in file_extensions)

def search_and_combine_files(directories, output_file):
    with open(output_file, 'w') as outfile:
        for directory in directories:
            for root, dirs, files in os.walk(directory):
                for file in files:
                    if is_code_file(file):
                        file_path = os.path.join(root, file)
                        with open(file_path, 'r', encoding='utf-8') as infile:
                            content = infile.read()
                            outfile.write(f'{file_path}\n\n')
                            outfile.write(f'Contenido:\n{content}\n\n{"-"*80}\n\n')

# Run the function
search_and_combine_files(directories, output_file)
print(f'Todos los archivos de código se han copiado en {output_file}')
