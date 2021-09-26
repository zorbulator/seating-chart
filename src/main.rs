use rsgenetic::pheno::*;
use rsgenetic::sim::*;
use rsgenetic::sim::seq::Simulator;
use rsgenetic::sim::select::*;
use std::cmp::Ordering;
use std::collections::HashMap;
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct SeatingFitness {
    value: isize
}


impl Fitness for SeatingFitness {
    fn zero() -> SeatingFitness {
        SeatingFitness { value: 0 }
    }

    fn abs_diff(&self, other: &SeatingFitness) -> SeatingFitness {
        SeatingFitness { value: isize::abs(self.value - other.value) }
    }
}

#[derive(Clone, Debug)]
struct Person {
    index: usize,
    name: String,
    preferences: Vec<isize>
}

#[derive(Clone)]
struct SeatingChart {
    people: Vec<Person>,
    group_size: usize
}

impl Phenotype<SeatingFitness> for SeatingChart {
    fn fitness(&self) -> SeatingFitness {
        let mut fitness = 0;

	// split people into their groups
        for group in self.people.chunks(self.group_size) {
            // for every pairing of two people add to the fitness how each person feels about the other
            for person in 0..self.group_size {
                for other_person in 0..self.group_size {
                    if person != other_person {
                        fitness += group[person].preferences[group[other_person].index];
                    }
                }
            }
        }

        SeatingFitness {value: fitness}
    }

    fn crossover(&self, other: &SeatingChart) -> Self {
        let num_people = self.people.len();
        // is this a slow way to do this because it requires an allocation?
        let mut remaining_indices: Vec<usize> = (0..num_people).collect();
        // alternate groups between both charts
        // if the two charts are [1,2,3,4] and [5,6,7,8] and the group size is 2 this should result in
        // [1,2,5,6,3,4,7,8]
        // also: iterator spaghetti warning
        let mut combined_people = self.people.chunks(self.group_size) // split into groups
                                  .zip(other.people.chunks(self.group_size)) // combine with other groups
                                  .map(|(a, b)| [a,b]) // put them into arrays, so now we have an array of arrays each containing two groups
                                  .flatten().flatten(); // flatten this twice to get a list of all of the people

        let mut final_chart: Vec<Person> = Vec::with_capacity(self.people.len());

        while remaining_indices.len() > 0 {
            let next_person = combined_people.next().expect("ran out of people to combine! this means not all indices are in use");
            if remaining_indices.contains(&next_person.index) {
                remaining_indices.retain(|&x| x != next_person.index);
                final_chart.push(next_person.clone());
            }
        }

        SeatingChart { people: final_chart, group_size: self.group_size }
    }

    fn mutate(&self) -> Self {
        let mut rng = ::rand::thread_rng();
        let mut new_chart = self.people.clone();
        let num_people = new_chart.len();
        new_chart.swap(rng.gen::<usize>() % num_people, rng.gen::<usize>() % num_people);
        SeatingChart { people: new_chart, group_size: self.group_size }
    }
}

#[derive(Debug)]
struct Preferences {
    names: Vec<String>,
    index: usize
}

fn main() {
    
    let mut reader = csv::Reader::from_reader(std::io::stdin());

    // replace with arguments or something?
    let positive_columns = vec![1,2];
    let negative_columns = vec![3];
    let pos_weight = 1;
    let neg_weight = -1;
    let group_size = 4;

    let mut all_people: Vec<Person> = Vec::new();
    let mut preference_map: HashMap<String, Preferences> = HashMap::new();

    let mut num_records = 0;

    for (i, result) in reader.records().enumerate() {
        let record = result.expect("invalid data record!");
        let person = Person { name: record[0].to_string(), index: i, preferences: Vec::new() };
        all_people.push(person);
        preference_map.insert(record[0].to_string(), Preferences { names: record.iter().skip(1).map(|s| s.to_string()).collect(), index: i });
        num_records += 1;
    }

    dbg!(&preference_map);

    for person in &mut all_people {
        person.preferences = vec![0; num_records];
        let preferences = preference_map.get(&person.name).expect("Person that should exist not found!");
        for (preference_index, preference) in preferences.names.iter().enumerate() {
            if let Some(named_person) = preference_map.get(preference) {
                let mut weight = 0;
                if positive_columns.contains(&(preference_index + 1)) { weight = pos_weight; }
                if negative_columns.contains(&(preference_index + 1)) { weight = neg_weight; }

                person.preferences[named_person.index] = weight;
            }
        }

        println!("{:?} ({})", person.preferences, person.name);
    }

    let mut population: Vec<SeatingChart> = Vec::with_capacity(100);
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        all_people.shuffle(&mut rng);
        population.push(SeatingChart { people: all_people.clone(), group_size: group_size });
    }

    let mut b = Simulator::builder(&mut population);
    b.with_selector(Box::new(StochasticSelector::new(10)))
        .with_max_iters(50000);
    let mut s = b.build();
    s.run();
    let result: &SeatingChart = s.get().unwrap();

    println!("final list: {:?}", result.people.iter().map(|p| &p.name).collect::<Vec<&String>>());
    println!("score: {}", result.fitness().value);
}
