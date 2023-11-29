REPO_FILES = Rake::FileList.new

def dirglob(pattern)
  result = Dir.glob(pattern, File::FNM_DOTMATCH)
  raise "No files found for pattern: #{pattern}" if result.empty?
  result
end

def filesize(*files)
  bytes = files.sum { |f| File.size(f) }
  (bytes / 1024.0 / 1024.0).round(2)
end

def repo_files
  REPO_FILES.include(`git ls-files`.split("\n")) if REPO_FILES.empty?
  REPO_FILES
end

def mtimes_for(regex)
  repo_files.each_with_object({}) do |path, mtimes|
    next unless path.match?(regex)
    mtimes[path] = File.mtime(path)
  end
end
