module ComponentsHelper
  def card(title: nil, icon: nil, icon_color: "text-neon-cyan", hover_border: nil, padding: nil, class: nil, &block)
    render partial: "components/card", locals: {
      title: title,
      icon: icon,
      icon_color: icon_color,
      hover_border: hover_border,
      padding: padding,
      extra_class: binding.local_variable_get(:class)
    }, &block
  end

  def empty_state(message:, icon: "◈", description: nil, action_text: nil, action_path: nil, turbo_frame: nil)
    render partial: "components/empty_state", locals: {
      icon: icon,
      message: message,
      description: description,
      action_text: action_text,
      action_path: action_path,
      turbo_frame: turbo_frame
    }
  end

  def badge(label:, variant: "default", size: "sm", dot: true)
    render partial: "components/badge", locals: {
      label: label,
      variant: variant,
      size: size,
      dot: dot
    }
  end

  def stat_card(label:, value:, icon: nil, icon_color: "text-neon-cyan", href: nil, hover_border: nil)
    render partial: "components/stat_card", locals: {
      label: label,
      value: value,
      icon: icon,
      icon_color: icon_color,
      href: href,
      hover_border: hover_border
    }
  end
end
