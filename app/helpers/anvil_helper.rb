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
end
