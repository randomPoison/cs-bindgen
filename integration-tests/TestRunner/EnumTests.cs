using System;
using System.Linq;
using Xunit;

namespace TestRunner
{
    public class EnumTests
    {
        [Fact]
        public void SimpleEnumRoundTrip()
        {
            foreach (var variant in Enum.GetValues(typeof(SimpleCEnum)).Cast<SimpleCEnum>())
            {
                SimpleCEnum result = IntegrationTests.RoundtripSimpleEnum(variant);
                Assert.Equal(variant, result);
            }
        }

        [Fact]
        public void DiscriminantEnumRoundTrip()
        {
            foreach (var variant in Enum.GetValues(typeof(EnumWithDiscriminants)).Cast<EnumWithDiscriminants>())
            {
                EnumWithDiscriminants result = IntegrationTests.RoundtripSimpleEnumWithDiscriminants(variant);
                Assert.Equal(variant, result);
            }
        }

        [Fact]
        public void GenerateDataEnum()
        {
            IDataEnum value = IntegrationTests.GenerateDataEnum();
            Baz baz = (Baz)value;
            Assert.Equal("Randal", baz.Name);
            Assert.Equal(11, baz.Value);
        }

        [Fact]
        public void DataEnumRoundTrip()
        {
            var foo = new Foo();
            Assert.Equal(foo, IntegrationTests.RoundtripDataEnum(foo));

            var bar = new Bar() { Element0 = "What a cool enum!" };
            Assert.Equal(bar, IntegrationTests.RoundtripDataEnum(bar));

            var baz = new Baz { Name = "Cool Guy McGee", Value = 69 };
            Assert.Equal(baz, IntegrationTests.RoundtripDataEnum(baz));
        }
    }
}
