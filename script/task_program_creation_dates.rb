#!/usr/bin/env ruby

=begin
This script traverses all the programs inside the LODA program rootdir.
It looks for all the LODA assembly programs there are.
This script determines date the program was first added to the git repository.

This script outputs a `bigram.csv` file, with this format:

    count;word0;word1
    18066;mov;mov
    16888;START;mov
    14712;mov;lpb
    13386;mov;sub
    13132;mov;add
    11776;add;mov
    10522;add;add
    9840;mul;add

=end

require 'csv'
require 'date'
require_relative 'config'

LODA_PROGRAM_ROOTDIR = Config.instance.loda_program_rootdir

output_filename = 'data/program_creation_dates.csv'

def relative_paths_for_all_programs(rootdir)
    Dir.glob(File.join("**", "*.asm"), base: rootdir).sort
end

def process_files(paths, csv)
    time_start = Process.clock_gettime(Process::CLOCK_MONOTONIC)
    progress_n = [(paths.count / 1000), 1].max
    number_of_rows = 0
    paths.each_with_index do |path, index|
        if (index % progress_n) == 0
            percent = (100 * index).to_f / paths.count
            percent_s = "%.2f" % percent
            
            time_end = Process.clock_gettime(Process::CLOCK_MONOTONIC)
            time_elapsed = time_end - time_start
            time_elapsed_s = "%.3f" % time_elapsed
            
            puts "progress: #{index}/#{paths.count}, %#{percent_s}  rows: #{number_of_rows}  elapsed: #{time_elapsed_s}"
        end
        path =~ /0*(\d+)[.]asm/
        program_id = $1.to_i
        if program_id == 0
            puts "Mismatch for #{filename}"
            next
        end
        
        # The last "creation date" of a file in a repository, and does it regardless of file renames/moves.
        # https://stackoverflow.com/questions/2390199/finding-the-date-time-a-file-was-first-added-to-a-git-repository
        output = `git log --diff-filter=A --follow --format=%aI -- #{path} | tail -1`
        # The output looks like this: "1984-12-30T20:12:09+01:00"
        
        success = $?.success?
        if !success
            puts "Unable to obtain git creation date for path: #{path}"
            next 
        end
        yyyymmdd = Date.iso8601(output.strip).strftime('%Y%m%d')
        if yyyymmdd == nil
            puts "Unable to parse as iso8601: '#{output}'  path: #{path}"
            next
        end
        csv << [program_id, yyyymmdd]
        csv.flush
        number_of_rows += 1
    end
    puts "number of rows written to csv file: #{number_of_rows}"
end

paths = relative_paths_for_all_programs(LODA_PROGRAM_ROOTDIR)
# paths = paths.first(50)
#p paths

CSV.open(output_filename, "wb", {:col_sep => ";"}) do |csv|
    csv << ["program id", "creation date"]
    Dir.chdir(LODA_PROGRAM_ROOTDIR) do
        process_files(paths, csv)
    end
end
