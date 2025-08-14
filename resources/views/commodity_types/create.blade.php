@extends('admin-layout.app')
@section('content')
   <div class="col-xl-12 box-col-6 p-3">
    <div class="row">
        <div class="col-md-9">
            <div class="card">
                <div class="card-body">
                    <h5 class="mb-3">Commodity Type Add</h5>
                    <form class="card-body" method="POST" action="{{ route('commodity_types.store') }}">
                        @csrf
                        <div class="row g-4">
                            <div class="col-md-6">
                                <div class="form-floating form-floating-outline">
                                    <input type="text" id="multicol-username" class="form-control" placeholder="Enter Name"
                                        name="name" />
                                    <label for="multicol-username">Name</label>
                                </div>
                            </div>
                        </div>
                        <div class="d-flex flex-row-reverse gap-1 mt-2">
                            <button type="submit" class="btn btn-outline-primary">Submit</button>
                            <a href="{{ route('commodity_types.index') }}" type="back" class="btn btn-outline-secondary">Back</a>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div> <!-- End of .row -->
</div>
@endsection
