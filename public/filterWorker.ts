interface ImageData {
  id: number;
  file_name: string;
  annotations: Array<{ category_id: number }>;
}

interface CategoryData {
  id: number;
  name: string;
}

interface FilterRequest {
  type: 'filter';
  images: ImageData[];
  categories: CategoryData[];
  search: string;
  filterClass: string | null;
}

interface FilterResponse {
  type: 'filterResult';
  filteredIndices: number[];
  payloadId: number;
}

type WorkerMessage = FilterRequest | FilterResponse;

let payloadIdCounter = 0;

self.onmessage = (event: MessageEvent<FilterRequest>) => {
  const { images, categories, search, filterClass, payloadId } = event.data;
  
  const categoryMap = new Map(categories.map(c => [c.id, c.name]));
  
  const filteredIndices: number[] = [];
  
  for (let i = 0; i < images.length; i++) {
    const img = images[i];
    
    // Search filter
    if (search && !img.file_name.toLowerCase().includes(search.toLowerCase())) {
      continue;
    }
    
    // Class filter
    if (filterClass) {
      const hasClass = img.annotations.some(ann => {
        const catName = categoryMap.get(ann.category_id);
        return catName === filterClass;
      });
      if (!hasClass) continue;
    }
    
    filteredIndices.push(i);
  }
  
  self.postMessage({
    type: 'filterResult',
    filteredIndices,
    payloadId: payloadId ?? payloadIdCounter++
  } as FilterResponse);
};

export {};