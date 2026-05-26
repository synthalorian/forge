module AnvilHelper
  def human_size(bytes)
    return "—" unless bytes.is_a?(Integer) || bytes.is_a?(Float)
    units = %w[B KB MB GB TB]
    size = bytes.to_f
    unit = units.shift
    while size >= 1024 && units.any?
      size /= 1024
      unit = units.shift
    end
    "#{size.round(1)} #{unit}"
  end

  def human_time(iso_string)
    return "—" if iso_string.nil?
    Time.parse(iso_string).localtime.strftime("%b %d, %Y %H:%M")
  rescue ArgumentError
    iso_string
  end

  def time_ago(iso_string)
    return "—" if iso_string.nil?
    time = Time.parse(iso_string).localtime
    seconds = (Time.now - time).to_i
    case seconds
    when 0...60 then "just now"
    when 60...3600 then "#{seconds / 60}m ago"
    when 3600...86_400 then "#{seconds / 3600}h ago"
    else "#{seconds / 86_400}d ago"
    end
  rescue ArgumentError
    iso_string
  end

  def backup_type_badge(type)
    variant = type == "full" ? "info" : "default"
    badge(label: type&.capitalize || "Unknown", variant: variant, size: "sm")
  end

  def relative_backup_time(iso_string)
    content_tag(:span, title: human_time(iso_string)) do
      time_ago(iso_string)
    end
  end

  # Build a nested tree structure from flat archive entries
  def build_file_tree(entries)
    tree = {}

    entries.each do |entry|
      path_parts = entry[:path].split("/")
      current = tree

      path_parts.each_with_index do |part, idx|
        current[part] ||= { children: {}, size: 0, is_dir: false, full_path: "" }
        if idx == path_parts.length - 1
          if entry[:directory]
            current[part][:is_dir] = true
            current[part][:full_path] = entry[:path] + "/"
          else
            current[part][:children] = nil
            current[part][:entry] = entry
            current[part][:size] = entry[:size].to_i
            current[part][:full_path] = entry[:path]
          end
        else
          current[part][:is_dir] = true
          current[part][:full_path] = path_parts[0..idx].join("/") + "/"
          current = current[part][:children]
        end
      end
    end

    # Calculate cumulative sizes for directories
    calculate_dir_sizes(tree)
    tree
  end

  def calculate_dir_sizes(node)
    total = 0
    node.each do |_name, data|
      if data[:children].nil?
        # Leaf file
        total += data[:size]
      else
        # Directory — recurse
        total += calculate_dir_sizes(data[:children])
      end
    end
    # Store cumulative size on each directory node
    node.each_value { |v| v[:size] = total if v[:children] }
    total
  end

  # Recursively render file tree nodes for the browse view
  def render_tree_nodes(node, indent_px, sort: true)
    items = node.to_a
    items = items.sort_by { |name, _| name.downcase } if sort

    output = +""
    output << %(<div class="space-y-0.5">)

    items.each do |name, data|
      next if name == "." # skip sentinel
      if data[:children].nil?
        # Leaf file
        size_label = data[:size] > 0 ? human_size(data[:size]) : ""
        output << %(<div class="flex items-center gap-2 py-0.5 hover:bg-bg-panel/50 rounded px-1 )
        output << %(     style="padding-left: #{indent_px}px">)
        output << %(  <span class="text-text-dim w-4 shrink-0 text-center"></span>)
        output << %(  <span class="shrink-0">📄</span>)
        output << %(  <span class="text-text-muted truncate">#{ERB::Util.html_escape(name)}</span>)
        output << %(  <span class="text-text-dim ml-auto shrink-0 font-mono">#{size_label}</span>)
        output << %(</div>)
      else
        # Directory
        child_count = data[:children].size
        sub_path = data[:full_path]
        size_label = data[:size] > 0 ? human_size(data[:size]) : ""

        output << %(<div class="flex items-center gap-2 py-0.5 hover:bg-bg-panel/50 rounded px-1 cursor-pointer )
        output << %(     style="padding-left: #{indent_px}px" )
        output << %(     data-action="click->file-tree#toggle" )
        output << %(     data-file-tree-target="toggle" )
        output << %(     data-dir-path="#{ERB::Util.html_escape(sub_path)}">)
        output << %(  <span class="file-tree-icon text-neon-cyan w-4 shrink-0 text-center">▾</span>)
        output << %(  <span class="shrink-0">📁</span>)
        output << %(  <span class="text-text-primary truncate">#{ERB::Util.html_escape(name)}/</span>)
        output << %(  <span class="text-text-dim ml-1">(#{child_count})</span>)
        output << %(  <span class="text-text-dim ml-auto shrink-0 font-mono">#{size_label}</span>)
        output << %(</div>)

        # Children container (starts expanded)
        output << %(<div data-dir-children="#{ERB::Util.html_escape(sub_path)}" data-file-tree-target="children">)
        output << render_tree_nodes(data[:children], indent_px + 16, sort: sort)
        output << %(</div>)
      end
    end

    output << %(</div>)
    output
  end
end
