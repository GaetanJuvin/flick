# frozen_string_literal: true

require_relative "lib/flick/version"

Gem::Specification.new do |spec|
  spec.name = "flick-ruby"
  spec.version = Flick::VERSION
  spec.authors = ["Gaetan Juvin"]
  spec.email = ["gaetanjuvin@gmail.com"]
  spec.summary = "Ruby SDK for the Flick feature flag platform"
  spec.description = "Polling-based SDK with local evaluation. Flipper-style API: Flick.enabled?(:flag, actor). Zero runtime dependencies."
  spec.homepage = "https://github.com/GaetanJUVIN/flick"
  spec.license = "MIT"
  spec.required_ruby_version = ">= 3.0"

  spec.metadata = {
    "source_code_uri" => "https://github.com/GaetanJUVIN/flick/tree/main/packages/sdk-ruby",
    "changelog_uri" => "https://github.com/GaetanJUVIN/flick/blob/main/packages/sdk-ruby/CHANGELOG.md",
    "rubygems_mfa_required" => "true"
  }

  spec.files = Dir["lib/**/*.rb"] + ["flick-ruby.gemspec"]
  spec.require_paths = ["lib"]

  # Zero runtime dependencies — stdlib only (net/http, json, uri)

  spec.add_development_dependency "rspec", "~> 3.13"
  spec.add_development_dependency "webmock", "~> 3.24"
end
