{{-- Breadcrumb Component Usage:
    @include('components.breadcrumb', [
        'title' => 'Page Title',
        'items' => [
            ['label' => 'Home', 'url' => route('dashboard'), 'icon' => 'home'],
            ['label' => 'Parent', 'url' => route('parent')],
            ['label' => 'Current Page']
        ]
    ])
--}}

@props(['title', 'items' => [], 'actions' => null])

<div class="page-title">
    <div class="row">
        <div class="col-12 col-md-6 mb-2 mb-md-0">
            <h4 class="mb-1">{{ $title }}</h4>
            @if(isset($subtitle))
                <p class="text-muted mb-0 small">{{ $subtitle }}</p>
            @endif
        </div>
        <div class="col-12 col-md-6">
            <div class="d-flex flex-column flex-md-row justify-content-md-between align-items-md-center gap-2">
                <!-- Breadcrumb -->
                <nav aria-label="breadcrumb" class="order-2 order-md-1">
                    <ol class="breadcrumb mb-0">
                        @foreach($items as $index => $item)
                            @if($index < count($items) - 1)
                                <li class="breadcrumb-item">
                                    @if(isset($item['url']))
                                        <a href="{{ $item['url'] }}">
                                            @if(isset($item['icon']))
                                                <i data-feather="{{ $item['icon'] }}" class="breadcrumb-icon"></i>
                                            @endif
                                            {{ $item['label'] }}
                                        </a>
                                    @else
                                        {{ $item['label'] }}
                                    @endif
                                </li>
                            @else
                                <li class="breadcrumb-item active" aria-current="page">
                                    {{ $item['label'] }}
                                </li>
                            @endif
                        @endforeach
                    </ol>
                </nav>

                <!-- Action Buttons -->
                @if($actions)
                    <div class="breadcrumb-actions order-1 order-md-2">
                        {{ $actions }}
                    </div>
                @endif
            </div>
        </div>
    </div>
</div>

@push('styles')
<style>
    .breadcrumb-icon {
        width: 14px;
        height: 14px;
        margin-right: 2px;
        vertical-align: middle;
    }
    
    .breadcrumb-actions .btn {
        font-size: 0.875rem;
        padding: 0.375rem 0.75rem;
    }
    
    @media (max-width: 767px) {
        .breadcrumb {
            font-size: 0.8rem;
        }
        
        .breadcrumb-actions {
            width: 100%;
        }
        
        .breadcrumb-actions .btn {
            width: 100%;
        }
    }
</style>
@endpush
