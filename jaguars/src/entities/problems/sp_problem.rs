use std::slice;
use std::sync::Arc;

use itertools::Itertools;
use ordered_float::NotNan;

use crate::collision_detection::hazards::filters::hazard_filter;
use crate::entities::bin::Bin;
use crate::entities::insertion_option::InsertionOption;
use crate::entities::instance::{Instance, PackingType};
use crate::entities::layout::Layout;
use crate::entities::placed_item_uid::PlacedItemUID;
use crate::entities::problems::problem::{LayoutIndex, Problem};
use crate::entities::problems::problem::private::ProblemPrivate;
use crate::entities::solution::Solution;
use crate::geometry::geo_traits::{Shape, Transformable};
use crate::util::assertions;
use crate::util::config::CDEConfig;

pub struct SPProblem {
    instance: Arc<Instance>,
    layout: Layout,
    strip_height: f64,
    strip_width: f64,
    missing_item_qtys: Vec<isize>,
    solution_id_counter: usize,
}

impl SPProblem {
    pub fn new(instance: Arc<Instance>, strip_width: f64, cde_config: CDEConfig) -> Self {
        match instance.packing_type() {
            PackingType::BinPacking(_) => panic!("cannot create StripPackingProblem from bin packing instance"),
            PackingType::StripPacking { height } => {
                let missing_item_qtys = instance.items().iter().map(|(_, qty)| *qty as isize).collect_vec();
                let strip_bin = Bin::from_strip(0, strip_width, *height, cde_config);
                let strip_height = *height;
                let layout = Layout::new(0, strip_bin);

                Self {
                    instance,
                    layout,
                    strip_height,
                    strip_width,
                    missing_item_qtys,
                    solution_id_counter: 0,
                }
            }
        }
    }

    pub fn modify_strip_width(&mut self, new_width: f64) {
        let old_p_uids = self.layout.placed_items().iter().map(|p_i| p_i.uid().clone()).collect_vec();
        self.missing_item_qtys.iter_mut().enumerate().for_each(|(i, qty)| *qty = self.instance.item_qty(i) as isize);
        let next_id = self.layout.id() + 1;
        self.layout = Layout::new(next_id, Bin::from_strip(next_id, new_width, self.strip_height, self.layout.bin().base_cde().config().clone()));
        self.strip_width = new_width;

        for p_uid in old_p_uids {
            let item = self.instance.item(p_uid.item_id());
            let entities_to_ignore = item.hazard_filter().map_or(vec![], |f| hazard_filter::ignored_entities(f, self.layout.cde().all_hazards()));
            let shape = item.shape();
            let transformation = p_uid.d_transformation().compose();
            if !self.layout.cde().surrogate_collides(shape.surrogate(), &transformation, entities_to_ignore.as_slice()) {
                let transformed_shape = shape.transform_clone(&transformation);
                if !self.layout.cde().poly_collides(&transformed_shape, entities_to_ignore.as_ref()) {
                    let insert_opt = InsertionOption::new(
                        LayoutIndex::Existing(0),
                        p_uid.item_id(),
                        transformation,
                        p_uid.d_transformation().clone(),
                    );
                    self.insert_item(&insert_opt);
                }
            }
        }
    }

    pub fn fit_strip_width(&mut self) {
        let max_x = self.layout.placed_items().iter()
            .map(|pi| pi.shape().bbox().x_max())
            .map(|x| NotNan::new(x).unwrap())
            .max().map_or(0.0, |x| x.into_inner());

        let strip_width = max_x + f32::EPSILON.sqrt() as f64;
        let n_items_in_old_strip = self.layout.placed_items().len();

        self.modify_strip_width(strip_width);

        assert_eq!(n_items_in_old_strip, self.layout.placed_items().len());
    }

    pub fn strip_height(&self) -> f64 {
        match self.instance.packing_type() {
            PackingType::BinPacking(_) => panic!("cannot get strip height from bin packing instance"),
            PackingType::StripPacking { height } => *height,
        }
    }

    pub fn strip_width(&self) -> f64 {
        self.strip_width
    }
}

impl Problem for SPProblem {
    fn insert_item(&mut self, i_opt: &InsertionOption) {
        assert_eq!(i_opt.layout_index(), &LayoutIndex::Existing(0), "strip packing problems only have a single layout");
        let item_id = i_opt.item_id();
        let item = self.instance.item(item_id);
        self.layout.place_item(item, i_opt.d_transformation());

        self.register_included_item(item_id);
    }

    fn remove_item(&mut self, layout_index: usize, pi_uid: &PlacedItemUID) {
        assert_eq!(layout_index, 0, "strip packing problems only have a single layout");
        self.layout.remove_item(pi_uid, false);
        self.unregister_included_item(pi_uid.item_id());
    }

    fn create_solution(&mut self, _old_solution: &Option<Solution>) -> Solution {
        let id = self.next_solution_id();
        let included_item_qtys = self.included_item_qtys();
        let bin_qtys = self.bin_qtys().to_vec();
        let stored_layouts = vec![self.layout.create_stored_layout()];
        let target_item_qtys = self.instance().items().iter().map(|(_, qty)| *qty).collect_vec();

        let solution = Solution::new(id, stored_layouts, self.usage(), included_item_qtys, target_item_qtys, bin_qtys);

        debug_assert!(assertions::problem_matches_solution(self, &solution));

        solution
    }

    fn restore_to_solution(&mut self, solution: &Solution) {
        debug_assert!(solution.stored_layouts().len() == 1);
        self.layout.restore(&solution.stored_layouts()[0], &self.instance);
        self.missing_item_qtys.iter_mut().enumerate().for_each(|(i, qty)| {
            *qty = (self.instance.item_qty(i) - solution.placed_item_qtys()[i]) as isize
        });

        debug_assert!(assertions::problem_matches_solution(self, solution));
    }

    fn instance(&self) -> &Arc<Instance> {
        &self.instance
    }

    fn layouts(&self) -> &[Layout] {
        slice::from_ref(&self.layout)
    }

    fn layouts_mut(&mut self) -> &mut [Layout] {
        slice::from_mut(&mut self.layout)
    }

    fn empty_layouts(&self) -> &[Layout] {
        &[]
    }

    fn missing_item_qtys(&self) -> &[isize] {
        &self.missing_item_qtys
    }

    fn empty_layout_has_stock(&self, _index: usize) -> bool {
        false
    }

    fn bin_qtys(&self) -> &[usize] {
        &[0]
    }
}


impl ProblemPrivate for SPProblem {
    fn next_solution_id(&mut self) -> usize {
        self.solution_id_counter += 1;
        self.solution_id_counter
    }

    fn missing_item_qtys_mut(&mut self) -> &mut [isize] {
        &mut self.missing_item_qtys
    }
}