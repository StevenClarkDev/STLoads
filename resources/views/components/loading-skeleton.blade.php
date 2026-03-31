{{-- Loading Skeleton Component Usage:
    @include('components.loading-skeleton', ['type' => 'table', 'rows' => 5])
    @include('components.loading-skeleton', ['type' => 'card'])
    @include('components.loading-skeleton', ['type' => 'list', 'items' => 3])
--}}

@props(['type' => 'table', 'rows' => 5, 'items' => 3, 'height' => null])

@if($type === 'table')
    <div class="skeleton-loader">
        <div class="table-responsive">
            <table class="table">
                <thead>
                    <tr>
                        @for($i = 0; $i < 6; $i++)
                            <th><div class="skeleton skeleton-text" style="width: {{ rand(60, 100) }}%;"></div></th>
                        @endfor
                    </tr>
                </thead>
                <tbody>
                    @for($r = 0; $r < $rows; $r++)
                        <tr>
                            @for($c = 0; $c < 6; $c++)
                                <td><div class="skeleton skeleton-text" style="width: {{ rand(40, 95) }}%;"></div></td>
                            @endfor
                        </tr>
                    @endfor
                </tbody>
            </table>
        </div>
    </div>

@elseif($type === 'card')
    <div class="skeleton-loader">
        <div class="card">
            <div class="card-body">
                <div class="skeleton skeleton-title mb-3"></div>
                <div class="skeleton skeleton-text mb-2"></div>
                <div class="skeleton skeleton-text mb-2" style="width: 80%;"></div>
                <div class="skeleton skeleton-text" style="width: 60%;"></div>
            </div>
        </div>
    </div>

@elseif($type === 'list')
    <div class="skeleton-loader">
        @for($i = 0; $i < $items; $i++)
            <div class="d-flex align-items-center mb-3 p-3 bg-white rounded">
                <div class="skeleton skeleton-avatar me-3"></div>
                <div class="flex-grow-1">
                    <div class="skeleton skeleton-text mb-2" style="width: 60%;"></div>
                    <div class="skeleton skeleton-text" style="width: 40%;"></div>
                </div>
            </div>
        @endfor
    </div>

@elseif($type === 'form')
    <div class="skeleton-loader">
        <div class="card">
            <div class="card-body">
                @for($i = 0; $i < 4; $i++)
                    <div class="mb-3">
                        <div class="skeleton skeleton-text mb-2" style="width: 120px;"></div>
                        <div class="skeleton skeleton-input"></div>
                    </div>
                @endfor
                <div class="skeleton skeleton-button"></div>
            </div>
        </div>
    </div>

@elseif($type === 'dashboard-widget')
    <div class="skeleton-loader">
        <div class="card small-widget">
            <div class="card-body">
                <div class="skeleton skeleton-text mb-2" style="width: 60%;"></div>
                <div class="skeleton skeleton-title" style="width: 40%;"></div>
            </div>
        </div>
    </div>

@endif

@once
@push('styles')
<style>
    /* Skeleton Loader Styles */
    .skeleton-loader {
        animation: skeleton-fade-in 0.3s ease-in;
    }
    
    .skeleton {
        background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
        background-size: 200% 100%;
        animation: skeleton-loading 1.5s ease-in-out infinite;
        border-radius: 4px;
    }
    
    .skeleton-text {
        height: 12px;
        margin-bottom: 8px;
    }
    
    .skeleton-title {
        height: 24px;
        width: 50%;
    }
    
    .skeleton-avatar {
        width: 48px;
        height: 48px;
        border-radius: 50%;
    }
    
    .skeleton-input {
        height: 38px;
        width: 100%;
    }
    
    .skeleton-button {
        height: 38px;
        width: 120px;
        margin-top: 16px;
    }
    
    @keyframes skeleton-loading {
        0% {
            background-position: 200% 0;
        }
        100% {
            background-position: -200% 0;
        }
    }
    
    @keyframes skeleton-fade-in {
        from {
            opacity: 0;
        }
        to {
            opacity: 1;
        }
    }
    
    /* Dark mode support */
    [data-theme="dark"] .skeleton {
        background: linear-gradient(90deg, #2a2a2a 25%, #3a3a3a 50%, #2a2a2a 75%);
        background-size: 200% 100%;
    }
</style>
@endpush
@endonce
