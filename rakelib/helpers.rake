def dirglob(pattern)
  result = Dir.glob(pattern, File::FNM_DOTMATCH)
  raise "No files found for pattern: #{pattern}" if result.empty?
  result
end

def filesize(*files)
  bytes = files.sum { |f| File.size(f) }
  (bytes / 1024.0 / 1024.0).round(2)
end
